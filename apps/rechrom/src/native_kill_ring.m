// Chromium blink/core/editing/kill_ring_mac.mm:30-84. AppKit exposes these
// private kill-ring functions without an NSKillRing header, just as in Blink.
#import <AppKit/AppKit.h>
#include <stdlib.h>

extern void _NSInitializeKillRing(void);
extern void _NSAppendToKillRing(NSString *);
extern void _NSPrependToKillRing(NSString *);
extern NSString *_NSYankFromKillRing(void);
extern void _NSNewKillRingSequence(void);
extern void _NSSetKillRingToYankedState(void);

// Page execution sequences may differ in this embedding. Serialize access to
// AppKit's single process ring, without giving interaction an OS thread owner.
static NSObject *KillRingLock(void) {
    static NSObject *lock;
    static dispatch_once_t once;
    dispatch_once(&once, ^{ lock = [NSObject new]; });
    return lock;
}
static void InitializeKillRingIfNeeded(void) {
    static bool initialized;
    if (!initialized) {
        initialized = true;
        _NSInitializeKillRing();
    }
}

bool rechrom_kill_ring_mutate(uint32_t operation, const uint16_t *text, size_t length) {
    @autoreleasepool {
        @synchronized(KillRingLock()) {
            @try {
                InitializeKillRingIfNeeded();
                switch (operation) {
                    case 0: _NSAppendToKillRing([[NSString alloc] initWithCharacters:text length:length]); break;
                    case 1: _NSPrependToKillRing([[NSString alloc] initWithCharacters:text length:length]); break;
                    case 2: _NSNewKillRingSequence(); break;
                    case 3: _NSSetKillRingToYankedState(); break;
                    default: return false;
                }
                return true;
            } @catch (NSException *exception) {
                return false;
            }
        }
    }
}

bool rechrom_kill_ring_yank(uint16_t **text, size_t *length) {
    *text = NULL;
    *length = 0;
    @autoreleasepool {
        @synchronized(KillRingLock()) {
            @try {
                InitializeKillRingIfNeeded();
                NSString *value = _NSYankFromKillRing();
                NSUInteger count = value.length;
                if (!count) return true;
                if (count > SIZE_MAX / sizeof(uint16_t)) return false;
                uint16_t *copy = malloc(count * sizeof(uint16_t));
                if (!copy) return false;
                [value getCharacters:copy range:NSMakeRange(0, count)];
                *text = copy;
                *length = count;
                return true;
            } @catch (NSException *exception) {
                return false;
            }
        }
    }
}
void rechrom_kill_ring_free(uint16_t *text) { free(text); }
