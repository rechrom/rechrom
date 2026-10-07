#import <AppKit/AppKit.h>
#import <WebKit/WebKit.h>

@interface BrowserDevToolsUIDelegate : NSObject <WKUIDelegate>
@end

@implementation BrowserDevToolsUIDelegate
- (void)webView:(WKWebView *)webView
    runOpenPanelWithParameters:(WKOpenPanelParameters *)parameters
    initiatedByFrame:(WKFrameInfo *)frame
    completionHandler:(void (^)(NSArray<NSURL *> *))completionHandler {
    NSOpenPanel *panel = [NSOpenPanel openPanel];
    panel.canChooseFiles = YES;
    panel.canChooseDirectories = NO;
    panel.allowsMultipleSelection = parameters.allowsMultipleSelection;
    [panel beginSheetModalForWindow:webView.window completionHandler:^(NSModalResponse result) {
        completionHandler(result == NSModalResponseOK ? panel.URLs : nil);
    }];
}
@end

// The host owns this developer window. Its frontend is independent of Page.
void browser_show_devtools(const char *address) {
    @autoreleasepool {
        NSCAssert([NSThread isMainThread], @"DevTools must open on the UI thread");
        static NSWindow *window;
        static WKWebView *webView;
        static BrowserDevToolsUIDelegate *uiDelegate;
        static NSString *loadedAddress;
        if (!window) {
            window = [[NSWindow alloc] initWithContentRect:NSMakeRect(0, 0, 1180, 780)
                styleMask:NSWindowStyleMaskTitled | NSWindowStyleMaskClosable |
                    NSWindowStyleMaskMiniaturizable | NSWindowStyleMaskResizable
                backing:NSBackingStoreBuffered defer:NO];
            window.title = @"DevTools — Rechrom";
            window.releasedWhenClosed = NO;
            window.minSize = NSMakeSize(600, 400);
            WKWebViewConfiguration *configuration = [[WKWebViewConfiguration alloc] init];
            configuration.websiteDataStore = [WKWebsiteDataStore nonPersistentDataStore];
            webView = [[WKWebView alloc] initWithFrame:window.contentView.bounds configuration:configuration];
            uiDelegate = [[BrowserDevToolsUIDelegate alloc] init];
            webView.UIDelegate = uiDelegate;
            webView.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;
            window.contentView = webView;
            [window center];
        }
        NSString *url = [NSString stringWithUTF8String:address];
        if (![loadedAddress isEqualToString:url]) {
            loadedAddress = [url copy];
            [webView loadRequest:[NSURLRequest requestWithURL:[NSURL URLWithString:url]]];
        }
        [window makeKeyAndOrderFront:nil];
        [NSApp activateIgnoringOtherApps:YES];
    }
}
