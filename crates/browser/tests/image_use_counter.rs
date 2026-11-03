#[path = "../src/test_thread.rs"]
mod test_thread;

use browser::page::Page;
use image_decoder::image_decoder::{DecodedImage, ImageDecodeInput, ImageDecoder};
use image_resource::{
    PaintImageAnimationInfo, PaintImageAnimationMetadata, PaintImageAnimationType,
};
use std::{cell::RefCell, io, rc::Rc, sync::Arc};
use url_loader::{URLLoadOperation, URLLoader, URLRequest, URLResponse};

struct Response(Option<URLResponse>);
impl URLLoadOperation for Response {
    fn Poll(&mut self) -> io::Result<Option<URLResponse>> {
        Ok(self.0.take())
    }
}

struct RepeatedAnimatedImageLoader;
impl URLLoader for RepeatedAnimatedImageLoader {
    fn Load(&mut self, request: &URLRequest) -> io::Result<Box<dyn URLLoadOperation>> {
        let document = request.destination == url_loader::RequestDestination::kDocument;
        Ok(Box::new(Response(Some(URLResponse {
            final_url: request.url.clone(),
            status_code: 200,
            mime_type: if document {
                "text/html".into()
            } else {
                "image/gif".into()
            },
            body: if document {
                br#"<!doctype html><style>body{margin:0}img{width:20px;height:20px}</style><img src=animated.gif><img src=animated.gif>"#.to_vec()
            } else {
                // The injected decoder owns producer facts in this test.
                vec![1]
            },
            ..Default::default()
        }))))
    }
}

#[derive(Default)]
struct AnimatedImageDecoder;
impl ImageDecoder for AnimatedImageDecoder {
    fn BackgroundFactory(&self) -> Option<Arc<dyn Fn() -> Box<dyn ImageDecoder> + Send + Sync>> {
        Some(Arc::new(|| Box::new(Self)))
    }

    fn Decode(&mut self, _input: &ImageDecodeInput<'_>) -> io::Result<DecodedImage> {
        Ok(DecodedImage {
            alpha_type: Default::default(),
            width: 1,
            height: 1,
            animation: PaintImageAnimationMetadata::Known(PaintImageAnimationInfo {
                animation_type: PaintImageAnimationType::Animated,
                repetition_count: 0,
                frame_count: 2,
            }),
            rgba8: vec![0, 0, 0, 255],
            ..Default::default()
        })
    }
}

// Chromium: core/layout/layout_image.cc:302-314. The loader and first
// LayoutImage are the baseline observers; painting the same animated resource
// through a second LayoutImage counts the document feature.
#[test]
fn painted_animated_image_used_more_than_once_counts_document_feature() {
    test_thread::run(|| {
        let _heap = foundation::LayoutHeapScope::new();
        let assembly = browser::CreateLayoutAssembly();
        let mut page = Page::Create(
            Rc::new(RefCell::new(RepeatedAnimatedImageLoader)),
            Rc::new(RefCell::new(AnimatedImageDecoder)),
            Rc::new(RefCell::new(browser::CreateSVGImageDecoder(&assembly))),
            browser::CreateBrowserConstraints(200, 100),
            None,
            None,
        );
        page.Open("https://animated-use-counter.test/", 16384, 4096)
            .unwrap();
        for _ in 0..100 {
            page.RunTask().unwrap();
            if page.CurrentFrame().is_some()
                && page
                    .Document()
                    .GetDocument()
                    .IsUseCounted(dom::WebFeature::kAnimatedImageUsedMoreThanOnce)
            {
                break;
            }
        }
        assert!(page
            .Document()
            .GetDocument()
            .IsUseCounted(dom::WebFeature::kAnimatedImageUsedMoreThanOnce));
    });
}
