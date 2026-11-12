// ui/base/clipboard/clipboard_mac.mm: ReadHTML, WriteHTML, WriteWebSmartPaste.
#import <AppKit/AppKit.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

typedef struct { const unsigned char *bytes; size_t length; } RechromClipboardBytes;
typedef struct {
  RechromClipboardBytes text, html, source_url;
  bool has_html, smart;
  RechromClipboardBytes png, svg, rtf, uris, custom;
  bool has_png, has_svg, has_rtf, has_uris;
} RechromClipboardValue;

static NSString *const SmartPasteType = @"dyn.ah62d4rv4gu8y63n2nuuhg5pbsm4ca6dbsr4gnkduqf31k3pcr7u1e3basv61a3k";
static NSString *const CustomType = @"org.chromium.web-custom-data";
static NSString *String(RechromClipboardBytes value) {
  return [[NSString alloc] initWithBytes:value.bytes length:value.length encoding:NSUTF8StringEncoding];
}
static RechromClipboardBytes CopyData(NSData *data) {
  void *bytes = malloc(MAX(data.length, 1));
  if (bytes && data.length) memcpy(bytes, data.bytes, data.length);
  return (RechromClipboardBytes){bytes, data.length};
}
static RechromClipboardBytes Copy(NSString *string) {
  return CopyData([(string ?: @"") dataUsingEncoding:NSUTF8StringEncoding]);
}
static NSData *Data(RechromClipboardBytes value) {
  return [NSData dataWithBytes:value.bytes length:value.length];
}
bool rechrom_clipboard_read(RechromClipboardValue *value) {
  @autoreleasepool {
    NSPasteboard *board = NSPasteboard.generalPasteboard;
    NSInteger sequence = board.changeCount;
    NSString *html = [board stringForType:NSPasteboardTypeHTML];
    value->text = Copy([board stringForType:NSPasteboardTypeString]);
    value->html = Copy(html);
    value->source_url = Copy([board stringForType:@"org.chromium.source-url"]);
    value->has_html = html != nil;
    value->smart = [board.types containsObject:SmartPasteType];
    NSData *png = [board dataForType:NSPasteboardTypePNG];
    if (!png) {
      NSImage *image = [[NSImage alloc] initWithPasteboard:board];
      NSBitmapImageRep *bitmap = image ? [[NSBitmapImageRep alloc] initWithData:image.TIFFRepresentation] : nil;
      png = [bitmap representationUsingType:NSBitmapImageFileTypePNG properties:@{}];
    }
    NSData *svg = [board dataForType:@"public.svg-image"];
    NSData *rtf = [board dataForType:NSPasteboardTypeRTF];
    NSMutableArray<NSString *> *uris = [NSMutableArray array];
    for (NSURL *url in [board readObjectsForClasses:@[NSURL.class] options:@{NSPasteboardURLReadingFileURLsOnlyKey:@YES}]) {
      [uris addObject:url.absoluteString];
    }
    if (!uris.count) {
      NSString *url = [board stringForType:NSPasteboardTypeURL];
      if (url) [uris addObject:url];
    }
    value->png = CopyData(png); value->svg = CopyData(svg); value->rtf = CopyData(rtf);
    value->uris = Copy([uris componentsJoinedByString:@"\r\n"]);
    value->custom = CopyData([board dataForType:CustomType]);
    value->has_png = png != nil; value->has_svg = svg != nil; value->has_rtf = rtf != nil; value->has_uris = uris.count != 0;
    return value->text.bytes && value->html.bytes && value->source_url.bytes && value->png.bytes && value->svg.bytes && value->rtf.bytes && value->uris.bytes && value->custom.bytes && board.changeCount == sequence;
  }
}
bool rechrom_clipboard_write(const RechromClipboardValue *value) {
  @autoreleasepool {
    NSPasteboardItem *item = [[NSPasteboardItem alloc] init];
    if (![item setString:String(value->text) forType:NSPasteboardTypeString]) return false;
    if (value->has_html) {
      if (![item setString:String(value->html) forType:NSPasteboardTypeHTML]) return false;
      if (value->source_url.length && ![item setString:String(value->source_url) forType:@"org.chromium.source-url"]) return false;
    }
    if (value->smart && ![item setData:NSData.data forType:SmartPasteType]) return false;
    if (value->has_png && ![item setData:Data(value->png) forType:NSPasteboardTypePNG]) return false;
    if (value->has_svg && ![item setData:Data(value->svg) forType:@"public.svg-image"]) return false;
    if (value->has_rtf && ![item setData:Data(value->rtf) forType:NSPasteboardTypeRTF]) return false;
    if (value->custom.length && ![item setData:Data(value->custom) forType:CustomType]) return false;
    NSMutableArray *objects = [NSMutableArray arrayWithObject:item];
    if (value->has_uris) {
      for (NSString *line in [String(value->uris) componentsSeparatedByCharactersInSet:NSCharacterSet.newlineCharacterSet]) {
        if (!line.length || [line hasPrefix:@"#"]) continue;
        NSURL *url = [NSURL URLWithString:line];
        if (!url) return false;
        if (url.isFileURL) [objects addObject:url];
        else if (![item setString:line forType:NSPasteboardTypeURL]) return false;
      }
    }
    NSPasteboard *board = NSPasteboard.generalPasteboard;
    [board clearContents];
    return [board writeObjects:objects];
  }
}
