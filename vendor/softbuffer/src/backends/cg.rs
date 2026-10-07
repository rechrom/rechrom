//! Softbuffer implementation using CoreGraphics.
#[cfg(all(target_os = "macos", feature = "native-iosurface"))]
#[path = "cg_iosurface.rs"]
mod iosurface;
use crate::backend_interface::*;
use crate::error::InitError;
#[cfg(not(all(target_os = "macos", feature = "native-iosurface")))]
use crate::util;
use crate::{Rect, SoftBufferError};
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Bool};
use objc2::{define_class, msg_send, AllocAnyThread, DefinedClass, MainThreadMarker, Message};
#[cfg(not(all(target_os = "macos", feature = "native-iosurface")))]
use objc2_core_foundation::CFRetained;
use objc2_core_foundation::CGPoint;
#[cfg(not(all(target_os = "macos", feature = "native-iosurface")))]
use objc2_core_graphics::{
    CGBitmapInfo, CGColorRenderingIntent, CGColorSpace, CGDataProvider, CGImage, CGImageAlphaInfo,
    CGImageByteOrderInfo, CGImageComponentInfo, CGImagePixelFormatInfo,
};
use objc2_foundation::{
    ns_string, NSDictionary, NSKeyValueChangeKey, NSKeyValueChangeNewKey,
    NSKeyValueObservingOptions, NSNumber, NSObject, NSObjectNSKeyValueObserverRegistration,
    NSString, NSValue,
};
use objc2_quartz_core::{kCAGravityTopLeft, CALayer, CATransaction};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawWindowHandle};

use std::ffi::c_void;
use std::marker::PhantomData;
#[cfg(not(all(target_os = "macos", feature = "native-iosurface")))]
use std::mem::size_of;
use std::num::NonZeroU32;
use std::ops::Deref;
use std::ptr;
#[cfg(not(all(target_os = "macos", feature = "native-iosurface")))]
use std::ptr::{slice_from_raw_parts_mut, NonNull};

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "SoftbufferObserver"]
    #[ivars = SendCALayer]
    #[derive(Debug)]
    struct Observer;

    /// NSKeyValueObserving
    impl Observer {
        #[unsafe(method(observeValueForKeyPath:ofObject:change:context:))]
        fn observe_value(
            &self,
            key_path: Option<&NSString>,
            _object: Option<&AnyObject>,
            change: Option<&NSDictionary<NSKeyValueChangeKey, AnyObject>>,
            _context: *mut c_void,
        ) {
            self.update(key_path, change);
        }
    }
);

impl Observer {
    fn new(layer: &CALayer) -> Retained<Self> {
        let this = Self::alloc().set_ivars(SendCALayer(layer.retain()));
        unsafe { msg_send![super(this), init] }
    }

    fn update(
        &self,
        key_path: Option<&NSString>,
        change: Option<&NSDictionary<NSKeyValueChangeKey, AnyObject>>,
    ) {
        let layer = self.ivars();

        let change =
            change.expect("requested a change dictionary in `addObserver`, but none was provided");
        let new = change
            .objectForKey(unsafe { NSKeyValueChangeNewKey })
            .expect("requested change dictionary did not contain `NSKeyValueChangeNewKey`");

        // NOTE: Setting these values usually causes a quarter second animation to occur, which is
        // undesirable.
        //
        // However, since we're setting them inside an observer, there already is a transaction
        // ongoing, and as such we don't need to wrap this in a `CATransaction` ourselves.

        if key_path == Some(ns_string!("contentsScale")) {
            let new = new.downcast::<NSNumber>().unwrap();
            let scale_factor = new.as_cgfloat();

            // Set the scale factor of the layer to match the root layer when it changes (e.g. if
            // moved to a different monitor, or monitor settings changed).
            layer.setContentsScale(scale_factor);
        } else if key_path == Some(ns_string!("bounds")) {
            let new = new.downcast::<NSValue>().unwrap();
            let bounds = new.get_rect().expect("new bounds value was not CGRect");

            // Set `bounds` and `position` so that the new layer is inside the superlayer.
            //
            // This differs from just setting the `bounds`, as it also takes into account any
            // translation that the superlayer may have that we'd want to preserve.
            layer.setFrame(bounds);
        } else {
            panic!("unknown observed keypath {key_path:?}");
        }
    }
}

#[derive(Debug)]
pub struct CGImpl<D, W> {
    /// Our layer.
    layer: SendCALayer,
    #[cfg(all(target_os = "macos", feature = "native-iosurface"))]
    targets: iosurface::TargetPool,
    /// The layer that our layer was created from.
    ///
    /// Can also be retrieved from `layer.superlayer()`.
    root_layer: SendCALayer,
    observer: Retained<Observer>,
    #[cfg(not(all(target_os = "macos", feature = "native-iosurface")))]
    color_space: CFRetained<CGColorSpace>,
    /// The width of the underlying buffer.
    width: usize,
    /// The height of the underlying buffer.
    height: usize,
    window_handle: W,
    _display: PhantomData<D>,
}

impl<D, W> Drop for CGImpl<D, W> {
    fn drop(&mut self) {
        // SAFETY: Registered in `new`, must be removed before the observer is deallocated.
        unsafe {
            self.root_layer
                .removeObserver_forKeyPath(&self.observer, ns_string!("contentsScale"));
            self.root_layer
                .removeObserver_forKeyPath(&self.observer, ns_string!("bounds"));
        }
    }
}

impl<D: HasDisplayHandle, W: HasWindowHandle> SurfaceInterface<D, W> for CGImpl<D, W> {
    type Context = D;
    type Buffer<'a>
        = BufferImpl<'a, D, W>
    where
        Self: 'a;

    fn new(window_src: W, _display: &D) -> Result<Self, InitError<W>> {
        // `NSView`/`UIView` can only be accessed from the main thread.
        let _mtm = MainThreadMarker::new().ok_or(SoftBufferError::PlatformError(
            Some("can only access Core Graphics handles from the main thread".to_string()),
            None,
        ))?;

        let root_layer = match window_src.window_handle()?.as_raw() {
            RawWindowHandle::AppKit(handle) => {
                // SAFETY: The pointer came from `WindowHandle`, which ensures that the
                // `AppKitWindowHandle` contains a valid pointer to an `NSView`.
                //
                // We use `NSObject` here to avoid importing `objc2-app-kit`.
                let view: &NSObject = unsafe { handle.ns_view.cast().as_ref() };

                // Force the view to become layer backed
                let _: () = unsafe { msg_send![view, setWantsLayer: Bool::YES] };

                // SAFETY: `-[NSView layer]` returns an optional `CALayer`
                let layer: Option<Retained<CALayer>> = unsafe { msg_send![view, layer] };
                layer.expect("failed making the view layer-backed")
            }
            RawWindowHandle::UiKit(handle) => {
                // SAFETY: The pointer came from `WindowHandle`, which ensures that the
                // `UiKitWindowHandle` contains a valid pointer to an `UIView`.
                //
                // We use `NSObject` here to avoid importing `objc2-ui-kit`.
                let view: &NSObject = unsafe { handle.ui_view.cast().as_ref() };

                // SAFETY: `-[UIView layer]` returns `CALayer`
                let layer: Retained<CALayer> = unsafe { msg_send![view, layer] };
                layer
            }
            _ => return Err(InitError::Unsupported(window_src)),
        };

        // Add a sublayer, to avoid interfering with the root layer, since setting the contents of
        // e.g. a view-controlled layer is brittle.
        let layer = CALayer::new();
        root_layer.addSublayer(&layer);

        // Set the anchor point and geometry. Softbuffer's uses a coordinate system with the origin
        // in the top-left corner.
        //
        // NOTE: This doesn't really matter unless we start modifying the `position` of our layer
        // ourselves, but it's nice to have in place.
        layer.setAnchorPoint(CGPoint::new(0.0, 0.0));
        layer.setGeometryFlipped(true);

        // Do not use auto-resizing mask.
        //
        // This is done to work around a bug in macOS 14 and above, where views using auto layout
        // may end up setting fractional values as the bounds, and that in turn doesn't propagate
        // properly through the auto-resizing mask and with contents gravity.
        //
        // Instead, we keep the bounds of the layer in sync with the root layer using an observer,
        // see below.
        //
        // layer.setAutoresizingMask(kCALayerHeightSizable | kCALayerWidthSizable);

        let observer = Observer::new(&layer);
        // Observe changes to the root layer's bounds and scale factor, and apply them to our layer.
        //
        // The previous implementation updated the scale factor inside `resize`, but this works
        // poorly with transactions, and is generally inefficient. Instead, we update the scale
        // factor only when needed because the super layer's scale factor changed.
        //
        // Note that inherent in this is an explicit design decision: We control the `bounds` and
        // `contentsScale` of the layer directly, and instead let the `resize` call that the user
        // controls only be the size of the underlying buffer.
        //
        // SAFETY: Observer deregistered in `Drop` before the observer object is deallocated.
        unsafe {
            root_layer.addObserver_forKeyPath_options_context(
                &observer,
                ns_string!("contentsScale"),
                NSKeyValueObservingOptions::New | NSKeyValueObservingOptions::Initial,
                ptr::null_mut(),
            );
            root_layer.addObserver_forKeyPath_options_context(
                &observer,
                ns_string!("bounds"),
                NSKeyValueObservingOptions::New | NSKeyValueObservingOptions::Initial,
                ptr::null_mut(),
            );
        }

        // Set the content so that it is placed in the top-left corner if it does not have the same
        // size as the surface itself.
        //
        // TODO(madsmtm): Consider changing this to `kCAGravityResize` to stretch the content if
        // resized to something that doesn't fit, see #177.
        layer.setContentsGravity(unsafe { kCAGravityTopLeft });

        // Initialize color space here, to reduce work later on.
        #[cfg(not(all(target_os = "macos", feature = "native-iosurface")))]
        let color_space = CGColorSpace::new_device_rgb().unwrap();

        // Grab initial width and height from the layer (whose properties have just been initialized
        // by the observer using `NSKeyValueObservingOptionInitial`).
        let size = layer.bounds().size;
        let scale_factor = layer.contentsScale();
        let width = (size.width * scale_factor) as usize;
        let height = (size.height * scale_factor) as usize;

        Ok(Self {
            layer: SendCALayer(layer),
            #[cfg(all(target_os = "macos", feature = "native-iosurface"))]
            targets: iosurface::TargetPool::default(),
            root_layer: SendCALayer(root_layer),
            observer,
            #[cfg(not(all(target_os = "macos", feature = "native-iosurface")))]
            color_space,
            width,
            height,
            _display: PhantomData,
            window_handle: window_src,
        })
    }

    #[inline]
    fn window(&self) -> &W {
        &self.window_handle
    }

    fn resize(&mut self, width: NonZeroU32, height: NonZeroU32) -> Result<(), SoftBufferError> {
        self.width = width.get() as usize;
        self.height = height.get() as usize;
        Ok(())
    }

    fn buffer_mut(&mut self) -> Result<BufferImpl<'_, D, W>, SoftBufferError> {
        Ok(BufferImpl {
            #[cfg(not(all(target_os = "macos", feature = "native-iosurface")))]
            buffer: util::PixelBuffer(vec![0; self.width * self.height]),
            #[cfg(all(target_os = "macos", feature = "native-iosurface"))]
            native: self.targets.acquire(self.width, self.height)?,
            imp: self,
        })
    }
}

#[cfg(all(target_os = "macos", feature = "native-iosurface"))]
impl<D, W> CGImpl<D, W> {
    pub(crate) fn prepared_buffer(&mut self) -> Result<PreparedBuffer, SoftBufferError> {
        Ok(PreparedBuffer {
            native: self.targets.acquire(self.width, self.height)?,
            width: NonZeroU32::new(self.width as u32).expect("non-empty CoreGraphics surface"),
            height: NonZeroU32::new(self.height as u32).expect("non-empty CoreGraphics surface"),
        })
    }

    pub(crate) fn present_prepared(
        &mut self,
        prepared: PreparedBuffer,
        callback: crate::PresentCallback,
        observer: Option<crate::NativeTransactionObserver>,
    ) -> Result<(), SoftBufferError> {
        let (surface, lease) = prepared.native.finish_with_lease()?;
        self.targets.submitted(surface.0.id());
        let pending = Box::new(MainThreadPresent {
            layer: SendCALayer(self.layer.0.clone()),
            surface,
            _lease: lease,
            callback,
            observer,
        });
        if MainThreadMarker::new().is_some() {
            pending.commit();
        } else {
            unsafe {
                dispatch_async_f(
                    std::ptr::addr_of!(_dispatch_main_q).cast_mut(),
                    Box::into_raw(pending).cast(),
                    commit_on_main_thread,
                );
            }
        }
        Ok(())
    }
}

/// CPU-complete IOSurface retained until the display scheduler submits it.
#[cfg(all(target_os = "macos", feature = "native-iosurface"))]
#[derive(Debug)]
pub(crate) struct PreparedBuffer {
    native: iosurface::Mapping,
    width: NonZeroU32,
    height: NonZeroU32,
}

#[cfg(all(target_os = "macos", feature = "native-iosurface"))]
impl PreparedBuffer {
    pub(crate) fn width(&self) -> NonZeroU32 { self.width }
    pub(crate) fn height(&self) -> NonZeroU32 { self.height }
    pub(crate) fn row_stride(&self) -> usize { self.native.row_stride() }
    pub(crate) fn pixels(&self) -> &[u32] { self.native.pixels() }
    pub(crate) fn pixels_mut(&mut self) -> &mut [u32] { self.native.pixels_mut() }
}

#[derive(Debug)]
pub struct BufferImpl<'a, D, W> {
    imp: &'a mut CGImpl<D, W>,
    #[cfg(not(all(target_os = "macos", feature = "native-iosurface")))]
    buffer: util::PixelBuffer,
    #[cfg(all(target_os = "macos", feature = "native-iosurface"))]
    native: iosurface::Mapping,
}

impl<D: HasDisplayHandle, W: HasWindowHandle> BufferInterface for BufferImpl<'_, D, W> {
    fn width(&self) -> NonZeroU32 {
        NonZeroU32::new(self.imp.width as u32).unwrap()
    }

    fn height(&self) -> NonZeroU32 {
        NonZeroU32::new(self.imp.height as u32).unwrap()
    }

    #[inline]
    fn pixels(&self) -> &[u32] {
        #[cfg(not(all(target_os = "macos", feature = "native-iosurface")))]
        {
            &self.buffer
        }
        #[cfg(all(target_os = "macos", feature = "native-iosurface"))]
        {
            self.native.pixels()
        }
    }

    #[inline]
    fn pixels_mut(&mut self) -> &mut [u32] {
        #[cfg(not(all(target_os = "macos", feature = "native-iosurface")))]
        {
            &mut self.buffer
        }
        #[cfg(all(target_os = "macos", feature = "native-iosurface"))]
        {
            self.native.pixels_mut()
        }
    }

    #[cfg(all(target_os = "macos", feature = "native-iosurface"))]
    fn pixel_format(&self) -> crate::BufferFormat {
        crate::BufferFormat::Argb8888
    }

    fn age(&self) -> u8 {
        0
    }

    #[cfg(all(target_os = "macos", feature = "native-iosurface"))]
    fn row_stride(&self) -> usize {
        self.native.row_stride()
    }

    #[cfg(not(all(target_os = "macos", feature = "native-iosurface")))]
    fn present(self) -> Result<(), SoftBufferError> {
        unsafe extern "C-unwind" fn release(
            _info: *mut c_void,
            data: NonNull<c_void>,
            size: usize,
        ) {
            let data = data.cast::<u32>();
            let slice = slice_from_raw_parts_mut(data.as_ptr(), size / size_of::<u32>());
            // SAFETY: This is the same slice that we passed to `Box::into_raw` below.
            drop(unsafe { Box::from_raw(slice) })
        }

        let timing = std::env::var_os("BROWSER_APP_PROFILE_PRESENT").is_some();
        let start = std::time::Instant::now();
        let data_provider = {
            let len = self.buffer.len() * size_of::<u32>();
            let buffer: *mut [u32] = Box::into_raw(self.buffer.0.into_boxed_slice());
            // Convert slice pointer to thin pointer.
            let data_ptr = buffer.cast::<c_void>();

            // SAFETY: The data pointer and length are valid.
            // The info pointer can safely be NULL, we don't use it in the `release` callback.
            unsafe {
                CGDataProvider::with_data(ptr::null_mut(), data_ptr, len, Some(release)).unwrap()
            }
        };

        let provider_done = std::time::Instant::now();
        // `CGBitmapInfo` consists of a combination of `CGImageAlphaInfo`, `CGImageComponentInfo`
        // `CGImageByteOrderInfo` and `CGImagePixelFormatInfo` (see e.g. `CGBitmapInfoMake`).
        //
        // TODO: Use `CGBitmapInfo::new` once the next version of objc2-core-graphics is released.
        let bitmap_info = CGBitmapInfo(
            CGImageAlphaInfo::NoneSkipFirst.0
                | CGImageComponentInfo::Integer.0
                | CGImageByteOrderInfo::Order32Little.0
                | CGImagePixelFormatInfo::Packed.0,
        );

        let image = unsafe {
            CGImage::new(
                self.imp.width,
                self.imp.height,
                8,
                32,
                self.imp.width * 4,
                Some(&self.imp.color_space),
                bitmap_info,
                Some(&data_provider),
                ptr::null(),
                false,
                CGColorRenderingIntent::RenderingIntentDefault,
            )
        }
        .unwrap();

        let image_done = std::time::Instant::now();
        // The CALayer has a default action associated with a change in the layer contents, causing
        // a quarter second fade transition to happen every time a new buffer is applied. This can
        // be avoided by wrapping the operation in a transaction and disabling all actions.
        CATransaction::begin();
        CATransaction::setDisableActions(true);
        let transaction_done = std::time::Instant::now();

        // SAFETY: The contents is `CGImage`, which is a valid class for `contents`.
        unsafe { self.imp.layer.setContents(Some(image.as_ref())) };

        let contents_done = std::time::Instant::now();
        CATransaction::commit();
        if timing {
            println!("cg-present provider_ms={:.3} image_ms={:.3} transaction_ms={:.3} contents_ms={:.3} commit_ms={:.3}",(provider_done-start).as_secs_f64()*1000.0,(image_done-provider_done).as_secs_f64()*1000.0,(transaction_done-image_done).as_secs_f64()*1000.0,(contents_done-transaction_done).as_secs_f64()*1000.0,contents_done.elapsed().as_secs_f64()*1000.0);
        }
        Ok(())
    }

    #[cfg(all(target_os = "macos", feature = "native-iosurface"))]
    fn present(self) -> Result<(), SoftBufferError> {
        let start = std::time::Instant::now();
        let surface = self.native.finish()?;
        let unlocked = std::time::Instant::now();
        CATransaction::begin();
        CATransaction::setDisableActions(true);
        self.imp.layer.setOpaque(true);
        // SAFETY: Chromium bridges IOSurfaceRef to id for CALayer.contents.
        // Core Animation retains the IOSurface; CPU mapping is already unlocked.
        unsafe {
            self.imp.layer.setContents(Some(
                &*(&*surface.0 as *const objc2_io_surface::IOSurfaceRef).cast::<AnyObject>(),
            ));
        }
        CATransaction::commit();
        self.imp.targets.submitted(surface.0.id());
        if std::env::var_os("BROWSER_APP_PROFILE_PRESENT").is_some() {
            println!(
                "iosurface-present unlock_ms={:.3} commit_ms={:.3} total_ms={:.3}",
                (unlocked - start).as_secs_f64() * 1000.0,
                unlocked.elapsed().as_secs_f64() * 1000.0,
                start.elapsed().as_secs_f64() * 1000.0
            );
        }
        Ok(())
    }

    #[cfg(all(target_os = "macos", feature = "native-iosurface"))]
    fn present_with_callback(self, callback: crate::PresentCallback,
        observer: Option<crate::NativeTransactionObserver>) -> Result<(), SoftBufferError> {
        let (surface, lease) = self.native.finish_with_lease()?;
        self.imp.targets.submitted(surface.0.id());
        let pending = Box::new(MainThreadPresent {
            layer: SendCALayer(self.imp.layer.0.clone()), surface,
            _lease: lease, callback, observer,
        });
        if MainThreadMarker::new().is_some() {
            pending.commit();
        } else {
            // SoftwareOutputDeviceMac -> HostDisplayClient ->
            // DisplayCALayerTree::GotIOSurfaceFrame: native view/layer updates
            // belong to the window main thread, independently of CPU painting.
            unsafe {
                dispatch_async_f(std::ptr::addr_of!(_dispatch_main_q).cast_mut(),
                    Box::into_raw(pending).cast(), commit_on_main_thread);
            }
        }
        Ok(())
    }

    fn present_with_damage(self, _damage: &[Rect]) -> Result<(), SoftBufferError> {
        self.present()
    }
}

#[cfg(all(target_os = "macos", feature = "native-iosurface"))]
struct MainThreadPresent {
    layer: SendCALayer,
    surface: iosurface::SendSurface,
    _lease: std::sync::Arc<()>,
    callback: crate::PresentCallback,
    observer: Option<crate::NativeTransactionObserver>,
}

#[cfg(all(target_os = "macos", feature = "native-iosurface"))]
impl MainThreadPresent {
    fn commit(self: Box<Self>) {
        let _main = MainThreadMarker::new().expect("IOSurface layer commit requires the main thread");
        let started = std::time::Instant::now();
        if let Some(observer) = &self.observer {
            register_transaction_observer(observer.clone(), self.surface.0.id());
        }
        CATransaction::begin();
        CATransaction::setDisableActions(true);
        self.layer.setOpaque(true);
        // SAFETY: IOSurface is a supported CALayer.contents object. The queued
        // lease remains live through this update and the native transaction.
        unsafe {
            self.layer.setContents(Some(
                &*(&*self.surface.0 as *const objc2_io_surface::IOSurfaceRef).cast::<AnyObject>(),
            ));
        }
        CATransaction::commit();
        (self.callback)(started, std::time::Instant::now());
    }
}

#[cfg(all(target_os = "macos", feature = "native-iosurface"))]
fn register_transaction_observer(observer: crate::NativeTransactionObserver, surface_id: u32) {
    use objc2::{sel, ClassType};
    // Chromium's CATransactionCoordinator uses this SPI. Registration observes
    // CA phases; it does not establish an implicit root or confirm presentation.
    // Register before opening our explicit update without changing its structure.
    // Each block owns immutable update identity; later presents cannot change it.
    let transaction_class = CATransaction::class();
    let supported: bool = unsafe {
        msg_send![transaction_class, respondsToSelector: sel!(addCommitHandler:forPhase:)]
    };
    if !supported { return; }
    for (phase, native_phase) in [(crate::NativeTransactionPhase::PreCommit, 1u32),
        (crate::NativeTransactionPhase::PostCommit, 2u32)] {
        let observer = observer.clone();
        let handler = block2::RcBlock::new(move || {
            observer(phase, surface_id, std::time::Instant::now());
        });
        // Core Animation copies and retains the block for its transaction.
        // The undocumented handler is best-effort diagnostic feedback only.
        unsafe { let _: () = msg_send![transaction_class, addCommitHandler: &*handler,
            forPhase: native_phase]; }
    }
}

#[cfg(all(target_os = "macos", feature = "native-iosurface"))]
unsafe extern "C" {
    // dispatch_get_main_queue() is a header macro for this global object.
    static _dispatch_main_q: c_void;
    fn dispatch_async_f(queue: *mut c_void, context: *mut c_void,
        work: unsafe extern "C" fn(*mut c_void));
}

#[cfg(all(target_os = "macos", feature = "native-iosurface"))]
unsafe extern "C" fn commit_on_main_thread(context: *mut c_void) {
    // SAFETY: dispatch_async_f invokes this exactly once for our owned Box.
    let pending = unsafe { Box::from_raw(context.cast::<MainThreadPresent>()) };
    pending.commit();
}

#[derive(Debug)]
struct SendCALayer(Retained<CALayer>);

// SAFETY: CALayer is dubiously thread safe, like most things in Core Animation.
// But since we make sure to do our changes within a CATransaction, it is
// _probably_ fine for us to use CALayer from different threads.
//
// See also:
// https://developer.apple.com/documentation/quartzcore/catransaction/1448267-lock?language=objc
// https://stackoverflow.com/questions/76250226/how-to-render-content-of-calayer-on-a-background-thread
unsafe impl Send for SendCALayer {}
// SAFETY: Same as above.
unsafe impl Sync for SendCALayer {}

impl Deref for SendCALayer {
    type Target = CALayer;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
