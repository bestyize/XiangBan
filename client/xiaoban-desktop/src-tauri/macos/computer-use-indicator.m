#import <AppKit/AppKit.h>
#import <CoreGraphics/CoreGraphics.h>
#import <QuartzCore/CALayer.h>

// Main-queue state. Order the badge relative to the target, including when another
// application's window covers it. Never activate the app or make a key window.
@interface XBComputerPanel : NSPanel
@end
@implementation XBComputerPanel
- (BOOL)canBecomeKeyWindow { return NO; }
- (BOOL)canBecomeMainWindow { return NO; }
@end

static XBComputerPanel *panel;
static NSTimer *timer;
static uint64_t currentToken;

static XBComputerPanel *makePanel(void) {
    NSRect frame = NSMakeRect(0, 0, 28, 24);
    XBComputerPanel *badge = [[XBComputerPanel alloc] initWithContentRect:frame
        styleMask:NSWindowStyleMaskBorderless | NSWindowStyleMaskNonactivatingPanel
        backing:NSBackingStoreBuffered defer:NO];
    badge.title = @"小伴正在操作此窗口";
    badge.floatingPanel = NO;
    badge.level = NSNormalWindowLevel;
    badge.hidesOnDeactivate = NO;
    badge.ignoresMouseEvents = YES;
    badge.excludedFromWindowsMenu = YES;
    badge.collectionBehavior = NSWindowCollectionBehaviorTransient |
        NSWindowCollectionBehaviorIgnoresCycle | NSWindowCollectionBehaviorFullScreenAuxiliary;
    badge.opaque = NO;
    badge.backgroundColor = NSColor.clearColor;
    badge.hasShadow = NO;
    NSVisualEffectView *background = [[NSVisualEffectView alloc] initWithFrame:frame];
    background.material = NSVisualEffectMaterialHUDWindow;
    background.state = NSVisualEffectStateActive;
    background.wantsLayer = YES;
    background.layer.cornerRadius = 7;
    background.layer.masksToBounds = YES;
    NSImageView *icon = [[NSImageView alloc] initWithFrame:NSInsetRect(frame, 5, 4)];
    icon.image = [NSImage imageWithSystemSymbolName:@"desktopcomputer"
                             accessibilityDescription:badge.title];
    icon.contentTintColor = NSColor.whiteColor;
    [background addSubview:icon];
    badge.contentView = background;
    return badge;
}

static void followWindow(CGWindowID target) {
    NSArray *windows = CFBridgingRelease(CGWindowListCopyWindowInfo(kCGWindowListOptionIncludingWindow, target));
    NSDictionary *window = windows.firstObject;
    CGRect bounds;
    NSScreen *primary = NSScreen.screens.firstObject;
    NSDictionary *rect = window[(__bridge NSString *)kCGWindowBounds];
    if (!primary || ![window[(__bridge NSString *)kCGWindowIsOnscreen] boolValue] ||
        !rect || !CGRectMakeWithDictionaryRepresentation((__bridge CFDictionaryRef)rect, &bounds) ||
        bounds.size.width < 40 || bounds.size.height < 40) {
        [panel orderOut:nil];
        return;
    }
    if (!panel) panel = makePanel();
    [panel setFrameTopLeftPoint:NSMakePoint(CGRectGetMinX(bounds) + 8,
        NSMaxY(primary.frame) - CGRectGetMinY(bounds) - 6)];
    [panel orderWindow:NSWindowAbove relativeTo:(NSInteger)target];
}

void xb_show_computer_indicator(uint32_t window, uint64_t token) {
    dispatch_async(dispatch_get_main_queue(), ^{
        [timer invalidate];
        currentToken = token;
        followWindow(window);
        timer = [NSTimer timerWithTimeInterval:0.15 repeats:YES block:^(NSTimer *tick) {
            (void)tick;
            followWindow(window);
        }];
        [NSRunLoop.mainRunLoop addTimer:timer forMode:NSRunLoopCommonModes];
    });
}

void xb_hide_computer_indicator(uint64_t token) {
    dispatch_async(dispatch_get_main_queue(), ^{
        if (token != currentToken) return;
        [timer invalidate];
        timer = nil;
        [panel orderOut:nil];
        currentToken = 0;
    });
}
