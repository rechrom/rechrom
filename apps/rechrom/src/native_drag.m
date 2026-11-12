// NSDraggingSource host leaf, matching WebContentsViewCocoa::startDragWithDropData
// and draggingSession:endedAtPoint:operation:. No DOM or drag policy lives here.
#import <AppKit/AppKit.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>
#include <stdlib.h>
#include <string.h>

typedef struct { const char *mime; const uint16_t *value; size_t length; } RechromDragString;
typedef struct {
    const uint16_t *name; size_t name_length; const char *mime; const char *path;
    const uint8_t *bytes; size_t length;
} RechromDragFile;
typedef struct {
    const uint8_t *rgba; size_t length; uint32_t width; uint32_t height; size_t row_bytes;
} RechromDragImage;
typedef void (*RechromDragCompletion)(void *, double, double, double, double, uint32_t);

typedef struct { uint8_t *bytes; size_t len; } RechromNativeString;

// Source-equivalent platform MIME lookup for net::GetMimeTypeFromFile. The
// returned UTF-8 buffer is owned by Rust and released with free(3).
RechromNativeString browser_drag_mime_from_extension(const uint8_t *extension, size_t len) {
    @autoreleasepool {
        RechromNativeString result = { NULL, 0 };
        if (!extension || !len) return result;
        NSString *value = [[NSString alloc] initWithBytes:extension
                                                   length:len
                                                 encoding:NSUTF8StringEncoding];
        if (!value) return result;
        NSString *mime = [UTType typeWithFilenameExtension:value].preferredMIMEType;
        if (!mime) return result;
        const char *utf8 = mime.UTF8String;
        if (!utf8) return result;
        result.len = strlen(utf8);
        result.bytes = malloc(result.len);
        if (!result.bytes) {
            result.len = 0;
            return result;
        }
        memcpy(result.bytes, utf8, result.len);
        return result;
    }
}

// Keep the primary URL/HTML/text flavors on the same item as file promises,
// matching WebDragSource's multi-flavor pasteboard writer.
@interface RechromFilePromise : NSFilePromiseProvider
@property(nonatomic, strong) NSPasteboardItem *extra;
@end
@implementation RechromFilePromise
- (NSArray<NSPasteboardType> *)writableTypesForPasteboard:(NSPasteboard *)pasteboard {
    NSMutableOrderedSet *types = [NSMutableOrderedSet orderedSetWithArray:[super writableTypesForPasteboard:pasteboard]];
    [types addObjectsFromArray:self.extra.types ?: @[]];
    return types.array;
}
- (id)pasteboardPropertyListForType:(NSPasteboardType)type {
    if ([self.extra.types containsObject:type]) return [self.extra propertyListForType:type];
    return [super pasteboardPropertyListForType:type];
}
@end

@interface RechromPromisedFile : NSObject <NSFilePromiseProviderDelegate>
@property(nonatomic, copy) NSString *name;
@property(nonatomic, strong) NSData *data;
@end
@implementation RechromPromisedFile
- (NSString *)filePromiseProvider:(NSFilePromiseProvider *)provider fileNameForType:(NSString *)type {
    return self.name;
}
- (void)filePromiseProvider:(NSFilePromiseProvider *)provider writePromiseToURL:(NSURL *)url
        completionHandler:(void (^)(NSError *))complete {
    NSError *error = nil;
    [self.data writeToURL:url options:NSDataWritingAtomic error:&error];
    complete(error);
}
@end

@interface RechromDragSource : NSObject <NSDraggingSource>
@property(nonatomic, strong) NSView *view;
@property(nonatomic, strong) NSArray *fileOwners;
@property(nonatomic) NSDragOperation operations;
@property(nonatomic) void *context;
@property(nonatomic) RechromDragCompletion complete;
@end
static RechromDragSource *activeDrag;
@implementation RechromDragSource
- (NSDragOperation)draggingSession:(NSDraggingSession *)session
        sourceOperationMaskForDraggingContext:(NSDraggingContext)context {
    return self.operations;
}
- (void)draggingSession:(NSDraggingSession *)session endedAtPoint:(NSPoint)point
        operation:(NSDragOperation)operation {
    NSPoint local = NSZeroPoint;
    if (self.view.window) {
        local = [self.view convertPoint:[self.view.window convertPointFromScreen:point] fromView:nil];
        if (!self.view.isFlipped) local.y = NSHeight(self.view.bounds)-local.y;
    }
    CGFloat screenTop = NSMaxY(NSScreen.screens.firstObject.frame);
    RechromDragCompletion complete = self.complete;
    void *context = self.context;
    self.context = NULL;
    self.complete = NULL;
    activeDrag = nil;
    if (complete) complete(context, local.x, local.y, point.x, screenTop-point.y,
        (uint32_t)(operation & (NSDragOperationCopy|NSDragOperationLink|NSDragOperationMove)));
}
@end

static NSString *RechromPasteboardType(NSString *mime) {
    if ([mime isEqualToString:@"text/plain"]) return NSPasteboardTypeString;
    if ([mime isEqualToString:@"text/html"]) return NSPasteboardTypeHTML;
    if ([mime isEqualToString:@"text/rtf"]) return NSPasteboardTypeRTF;
    return mime;
}

bool rechrom_start_native_drag(void *view_pointer, const RechromDragString *strings, size_t strings_len,
        const RechromDragFile *files, size_t files_len, const RechromDragImage *image,
        int32_t offset_x, int32_t offset_y, uint32_t operation_mask,
        void *context, RechromDragCompletion complete) {
    @autoreleasepool {
        NSCAssert(NSThread.isMainThread, @"native drag starts on the AppKit sequence");
        NSView *view = (__bridge NSView *)view_pointer;
        if (!view.window || activeDrag || !complete) return false;
        NSMutableArray<NSDraggingItem *> *items = [NSMutableArray array];
        NSMutableArray *fileOwners = [NSMutableArray array];
        NSPasteboardItem *text = [[NSPasteboardItem alloc] init];
        for (size_t i=0; i<strings_len; ++i) {
            NSString *mime = [NSString stringWithUTF8String:strings[i].mime];
            NSString *value = [[NSString alloc] initWithCharacters:strings[i].value length:strings[i].length];
            if (!mime || !value) return false;
            [text setString:value forType:RechromPasteboardType(mime)];
            // Keep arbitrary MIME data as well as the OS standard aliases.
            if (![mime isEqualToString:RechromPasteboardType(mime)]) [text setString:value forType:mime];
            if ([mime isEqualToString:@"text/uri-list"]) {
                for (NSString *line in [value componentsSeparatedByCharactersInSet:NSCharacterSet.newlineCharacterSet]) {
                    if (line.length && ![line hasPrefix:@"#"]) {
                        [text setString:line forType:NSPasteboardTypeURL];
                        break;
                    }
                }
            }
        }
        if (text.types.count && !files_len) [items addObject:[[NSDraggingItem alloc] initWithPasteboardWriter:text]];
        for (size_t i=0; i<files_len; ++i) {
            id<NSPasteboardWriting> writer = nil;
            if (files[i].path) {
                NSString *path = [NSString stringWithUTF8String:files[i].path];
                if (!path) return false;
                NSPasteboardItem *item = i == 0 ? text : [[NSPasteboardItem alloc] init];
                [item setString:[NSURL fileURLWithPath:path].absoluteString forType:NSPasteboardTypeFileURL];
                writer = item;
            } else {
                NSString *name = [[NSString alloc] initWithCharacters:files[i].name length:files[i].name_length];
                NSString *mime = [NSString stringWithUTF8String:files[i].mime];
                if (!name || !mime) return false;
                RechromPromisedFile *file = [[RechromPromisedFile alloc] init];
                file.name = name.lastPathComponent.length ? name.lastPathComponent : @"download";
                file.data = [NSData dataWithBytes:files[i].bytes length:files[i].length];
                UTType *type = [UTType typeWithMIMEType:mime];
                if (!type && file.name.pathExtension.length) type = [UTType typeWithFilenameExtension:file.name.pathExtension];
                NSString *identifier = (type ?: UTTypeData).identifier;
                RechromFilePromise *provider = [[RechromFilePromise alloc]
                    initWithFileType:identifier delegate:file];
                provider.extra = i == 0 ? text : [[NSPasteboardItem alloc] init];
                [provider.extra setData:file.data forType:identifier];
                [provider.extra setData:file.data forType:mime];
                [fileOwners addObject:file];
                writer = provider;
            }
            [items addObject:[[NSDraggingItem alloc] initWithPasteboardWriter:writer]];
        }
        if (!items.count) return false;
        NSImage *preview = nil;
        if (image) {
            NSBitmapImageRep *rep = [[NSBitmapImageRep alloc] initWithBitmapDataPlanes:NULL
                pixelsWide:image->width pixelsHigh:image->height bitsPerSample:8 samplesPerPixel:4
                hasAlpha:YES isPlanar:NO colorSpaceName:NSDeviceRGBColorSpace
                bytesPerRow:image->row_bytes bitsPerPixel:32];
            if (!rep) return false;
            memcpy(rep.bitmapData, image->rgba, image->row_bytes*image->height);
            CGFloat scale = view.window.backingScaleFactor;
            preview = [[NSImage alloc] initWithSize:NSMakeSize(image->width/scale,image->height/scale)];
            [preview addRepresentation:rep];
        } else {
            preview = [NSImage imageNamed:NSImageNameMultipleDocuments];
        }
        NSPoint mouse = view.window.mouseLocationOutsideOfEventStream;
        // Chromium also synthesizes a drag NSEvent when renderer startup
        // returns asynchronously, using the native pointer's current position.
        NSEvent *event = [NSEvent mouseEventWithType:NSEventTypeLeftMouseDragged location:mouse
            modifierFlags:NSApp.currentEvent.modifierFlags timestamp:NSProcessInfo.processInfo.systemUptime
            windowNumber:view.window.windowNumber context:nil eventNumber:0 clickCount:1 pressure:1.0];
        mouse = [view convertPoint:mouse fromView:nil];
        NSRect rect = NSMakeRect(mouse.x-offset_x,
            view.isFlipped ? mouse.y-offset_y : mouse.y-preview.size.height+offset_y,
            preview.size.width, preview.size.height);
        for (NSDraggingItem *item in items) [item setDraggingFrame:rect contents:preview];
        RechromDragSource *source = [[RechromDragSource alloc] init];
        source.view = view;
        source.fileOwners = fileOwners;
        source.operations = operation_mask & (NSDragOperationCopy|NSDragOperationLink|NSDragOperationMove);
        source.context = context;
        source.complete = complete;
        activeDrag = source;
        @try {
            NSDraggingSession *session = [view beginDraggingSessionWithItems:items event:event source:source];
            if (session) return true;
        } @catch (NSException *exception) {
            // Propagate refusal as a typed failed completion on the Rust side.
        }
        activeDrag = nil;
        source.context = NULL;
        source.complete = NULL;
        return false;
    }
}
