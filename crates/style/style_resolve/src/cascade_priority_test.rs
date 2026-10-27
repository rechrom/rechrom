// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/resolver/cascade_priority_test.cc:12-525
#![allow(non_snake_case)]
use super::*;
use CascadeOrigin as Origin;

// cpp: cascade_priority_test.cc:12-48
#[derive(Clone, Copy)]
struct Options {
    origin: CascadeOrigin,
    important: bool,
    tree_order: u16,
    is_inline_style: bool,
    is_try_style: bool,
    is_try_tactics_style: bool,
    layer_order: u16,
    rule_index: u16,
    declaration_index: u16,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            origin: Origin::kAuthor,
            important: false,
            tree_order: 0,
            is_inline_style: false,
            is_try_style: false,
            is_try_tactics_style: false,
            layer_order: 0,
            rule_index: 0,
            declaration_index: 0,
        }
    }
}
fn Priority(o: Options) -> CascadePriority {
    CascadePriority::FromParts(
        o.origin,
        o.important,
        o.tree_order,
        o.is_inline_style,
        o.is_try_style,
        o.is_try_tactics_style,
        o.layer_order,
        o.rule_index,
        o.declaration_index,
    )
}
fn AuthorPriority(tree_order: u16, rule_index: u16, declaration_index: u16) -> CascadePriority {
    Priority(Options {
        tree_order,
        rule_index,
        declaration_index,
        ..Options::default()
    })
}
fn ImportantAuthorPriority(
    tree_order: u16,
    rule_index: u16,
    declaration_index: u16,
) -> CascadePriority {
    Priority(Options {
        important: true,
        tree_order,
        rule_index,
        declaration_index,
        ..Options::default()
    })
}
// cpp: cascade_priority_test.cc:154-156
const ALL_ORIGINS: [CascadeOrigin; 5] = [
    Origin::kUserAgent,
    Origin::kUser,
    Origin::kAuthor,
    Origin::kTransition,
    Origin::kAnimation,
];

// cpp: cascade_priority_test.cc:51-61
#[test]
fn TestEncodeOriginImportance() {
    assert_eq!(0b00001, EncodeOriginImportance(Origin::kUserAgent, false));
    assert_eq!(0b00010, EncodeOriginImportance(Origin::kUser, false));
    assert_eq!(0b00100, EncodeOriginImportance(Origin::kAuthor, false));
    assert_eq!(0b00101, EncodeOriginImportance(Origin::kAnimation, false));
    assert_eq!(0b01011, EncodeOriginImportance(Origin::kAuthor, true));
    assert_eq!(0b01101, EncodeOriginImportance(Origin::kUser, true));
    assert_eq!(0b01110, EncodeOriginImportance(Origin::kUserAgent, true));
    assert_eq!(0b10000, EncodeOriginImportance(Origin::kTransition, false));
}

// cpp: cascade_priority_test.cc:63-98
#[test]
fn TestOriginOperators() {
    let priorities = [
        Priority(Options {
            origin: CascadeOrigin::kTransition,
            ..Options::default()
        }),
        Priority(Options {
            origin: CascadeOrigin::kAnimation,
            ..Options::default()
        }),
        Priority(Options {
            origin: CascadeOrigin::kAuthor,
            ..Options::default()
        }),
        Priority(Options {
            origin: CascadeOrigin::kUser,
            ..Options::default()
        }),
        Priority(Options {
            origin: CascadeOrigin::kUserAgent,
            ..Options::default()
        }),
        Priority(Options {
            origin: CascadeOrigin::kNone,
            ..Options::default()
        }),
    ];

    for i in 0..priorities.len() {
        for j in i..priorities.len() {
            assert!(priorities[i] >= priorities[j]);
            assert!(!(priorities[i] < priorities[j]));
        }
    }

    for i in 0..priorities.len() {
        for j in i + 1..priorities.len() {
            assert!(priorities[j] < priorities[i]);
            assert!(!(priorities[j] >= priorities[i]));
        }
    }

    for priority in priorities {
        assert_eq!(priority, priority);
    }

    for i in 0..priorities.len() {
        for j in 0..priorities.len() {
            if i == j {
                continue;
            }
            assert_ne!(priorities[i], priorities[j]);
        }
    }
}

// cpp: cascade_priority_test.cc:100-117
#[test]
fn TestOriginImportance() {
    let priorities = [
        Priority(Options {
            origin: CascadeOrigin::kTransition,
            important: false,
            ..Options::default()
        }),
        Priority(Options {
            origin: CascadeOrigin::kUserAgent,
            important: true,
            ..Options::default()
        }),
        Priority(Options {
            origin: CascadeOrigin::kUser,
            important: true,
            ..Options::default()
        }),
        Priority(Options {
            origin: CascadeOrigin::kAuthor,
            important: true,
            ..Options::default()
        }),
        Priority(Options {
            origin: CascadeOrigin::kAnimation,
            important: false,
            ..Options::default()
        }),
        Priority(Options {
            origin: CascadeOrigin::kAuthor,
            important: false,
            ..Options::default()
        }),
        Priority(Options {
            origin: CascadeOrigin::kUser,
            important: false,
            ..Options::default()
        }),
        Priority(Options {
            origin: CascadeOrigin::kUserAgent,
            important: false,
            ..Options::default()
        }),
        Priority(Options {
            origin: CascadeOrigin::kNone,
            important: false,
            ..Options::default()
        }),
    ];

    for i in 0..priorities.len() {
        for j in i..priorities.len() {
            assert!(priorities[i] >= priorities[j]);
        }
    }
}

// cpp: cascade_priority_test.cc:119-152
#[test]
fn TestIsImportant() {
    assert!(
        !(Priority(Options {
            origin: Origin::kUserAgent,
            ..Options::default()
        })
        .IsImportant())
    );
    assert!(
        !(Priority(Options {
            origin: Origin::kUser,
            ..Options::default()
        })
        .IsImportant())
    );
    assert!(
        !(Priority(Options {
            origin: Origin::kAuthor,
            ..Options::default()
        })
        .IsImportant())
    );
    assert!(
        !(Priority(Options {
            origin: Origin::kAnimation,
            ..Options::default()
        })
        .IsImportant())
    );
    assert!(
        !(Priority(Options {
            origin: Origin::kTransition,
            ..Options::default()
        })
        .IsImportant())
    );
    assert!(
        !(Priority(Options {
            origin: Origin::kAuthor,
            important: false,
            tree_order: 1024,
            layer_order: 2048,
            rule_index: 4096,
            declaration_index: 8192,
            ..Options::default()
        })
        .IsImportant())
    );

    assert!(Priority(Options {
        origin: Origin::kUserAgent,
        important: true,
        ..Options::default()
    })
    .IsImportant());
    assert!(Priority(Options {
        origin: Origin::kUser,
        important: true,
        ..Options::default()
    })
    .IsImportant());
    assert!(Priority(Options {
        origin: Origin::kAuthor,
        important: true,
        ..Options::default()
    })
    .IsImportant());
    assert!(Priority(Options {
        origin: Origin::kAnimation,
        important: true,
        ..Options::default()
    })
    .IsImportant());
    assert!(Priority(Options {
        origin: Origin::kTransition,
        important: true,
        ..Options::default()
    })
    .IsImportant());
    assert!(Priority(Options {
        origin: Origin::kAuthor,
        important: true,
        tree_order: 1024,
        layer_order: 2048,
        rule_index: 4096,
        declaration_index: 8192,
        ..Options::default()
    })
    .IsImportant());
}

// cpp: cascade_priority_test.cc:158-174
#[test]
fn TestGetOrigin() {
    for origin in ALL_ORIGINS {
        assert_eq!(
            Priority(Options {
                origin: origin,
                important: false,
                ..Options::default()
            })
            .GetOrigin(),
            origin
        );
    }

    for origin in ALL_ORIGINS {
        if origin == CascadeOrigin::kAnimation {
            continue;
        }
        if origin == CascadeOrigin::kTransition {
            continue;
        }
        assert_eq!(
            Priority(Options {
                origin: origin,
                important: true,
                ..Options::default()
            })
            .GetOrigin(),
            origin
        );
    }
}

// cpp: cascade_priority_test.cc:176-185
#[test]
fn TestHasOrigin() {
    for origin in ALL_ORIGINS {
        if origin != CascadeOrigin::kNone {
            assert!(CascadePriority::FromOrigin(origin).HasOrigin());
        } else {
            assert!(!(CascadePriority::FromOrigin(origin).HasOrigin()));
        }
    }
    assert!(!(CascadePriority::new().HasOrigin()));
}

// cpp: cascade_priority_test.cc:187-199
#[test]
fn TestEncodeTreeOrder() {
    assert_eq!(0, EncodeTreeOrder(0, false));
    assert_eq!(1, EncodeTreeOrder(1, false));
    assert_eq!(2, EncodeTreeOrder(2, false));
    assert_eq!(100, EncodeTreeOrder(100, false));
    assert_eq!(0xFFFF, EncodeTreeOrder(0xFFFF, false));

    assert_eq!(0 ^ 0xFFFF, EncodeTreeOrder(0, true));
    assert_eq!(1 ^ 0xFFFF, EncodeTreeOrder(1, true));
    assert_eq!(2 ^ 0xFFFF, EncodeTreeOrder(2, true));
    assert_eq!(100 ^ 0xFFFF, EncodeTreeOrder(100, true));
    assert_eq!(0xFFFF ^ 0xFFFF, EncodeTreeOrder(0xFFFF, true));
}

// cpp: cascade_priority_test.cc:201-208
#[test]
fn TestTreeOrder() {
    let origin = CascadeOrigin::kAuthor;
    assert!(
        CascadePriority::FromOriginImportanceTreeOrder(origin, false, 1)
            >= CascadePriority::FromOriginImportanceTreeOrder(origin, false, 0)
    );
    assert!(
        CascadePriority::FromOriginImportanceTreeOrder(origin, false, 7)
            >= CascadePriority::FromOriginImportanceTreeOrder(origin, false, 6)
    );
    assert!(
        CascadePriority::FromOriginImportanceTreeOrder(origin, false, 42)
            >= CascadePriority::FromOriginImportanceTreeOrder(origin, false, 42)
    );
    assert!(
        !(CascadePriority::FromOriginImportanceTreeOrder(origin, false, 1)
            >= CascadePriority::FromOriginImportanceTreeOrder(origin, false, 8))
    );
}

// cpp: cascade_priority_test.cc:210-217
#[test]
fn TestTreeOrderImportant() {
    let origin = CascadeOrigin::kAuthor;
    assert!(
        CascadePriority::FromOriginImportanceTreeOrder(origin, true, 0)
            >= CascadePriority::FromOriginImportanceTreeOrder(origin, true, 1)
    );
    assert!(
        CascadePriority::FromOriginImportanceTreeOrder(origin, true, 6)
            >= CascadePriority::FromOriginImportanceTreeOrder(origin, true, 7)
    );
    assert!(
        CascadePriority::FromOriginImportanceTreeOrder(origin, true, 42)
            >= CascadePriority::FromOriginImportanceTreeOrder(origin, true, 42)
    );
    assert!(
        !(CascadePriority::FromOriginImportanceTreeOrder(origin, true, 8)
            >= CascadePriority::FromOriginImportanceTreeOrder(origin, true, 1))
    );
}

// cpp: cascade_priority_test.cc:219-226
#[test]
fn TestTreeOrderDifferentOrigin() {
    // Tree order does not matter if the origin is different.
    let author = CascadeOrigin::kAuthor;
    let transition = CascadeOrigin::kTransition;
    assert!(
        CascadePriority::FromOriginImportance(transition, true)
            >= CascadePriority::FromOriginImportance(author, true)
    );
    assert!(
        CascadePriority::FromOriginImportance(transition, true)
            >= CascadePriority::FromOriginImportance(author, true)
    );
}

// cpp: cascade_priority_test.cc:228-237
#[test]
fn TestPosition() {
    // AuthorPriority(tree_order, rule_index, declaration_index)
    assert!(AuthorPriority(0, 0, 0) >= AuthorPriority(0, 0, 0));
    assert!(AuthorPriority(0, 0, 1) >= AuthorPriority(0, 0, 1));
    assert!(AuthorPriority(0, 0, 1) >= AuthorPriority(0, 0, 0));
    assert!(AuthorPriority(0, 0, 2) >= AuthorPriority(0, 0, 1));
    assert!(AuthorPriority(0, 0xFFFF, 0xFFFF) >= AuthorPriority(0, 0xFFFF, 0xFFFE));
    assert!(!(AuthorPriority(0, 0, 2) >= AuthorPriority(0, 0, 3)));
}

// cpp: cascade_priority_test.cc:239-245
#[test]
fn TestPositionAndTreeOrder() {
    // AuthorPriority(tree_order, rule_index, declaration_index)
    assert!(AuthorPriority(1, 0, 0) >= AuthorPriority(0, 0, 0));
    assert!(AuthorPriority(1, 0, 1) >= AuthorPriority(0, 0, 1));
    assert!(AuthorPriority(1, 0, 1) >= AuthorPriority(0, 0, 3));
    assert!(AuthorPriority(1, 0, 2) >= AuthorPriority(0, 0xFFFF, 0xFFFF));
}

// cpp: cascade_priority_test.cc:247-254
#[test]
fn TestPositionAndOrigin() {
    // [Important]AuthorPriority(tree_order, rule_index, declaration_index)
    assert!(ImportantAuthorPriority(0, 0, 0) >= AuthorPriority(0, 0, 0));
    assert!(ImportantAuthorPriority(0, 0, 1) >= AuthorPriority(0, 0, 1));
    assert!(ImportantAuthorPriority(0, 0, 1) >= AuthorPriority(0, 0, 3));
    assert!(ImportantAuthorPriority(0, 0, 2) >= AuthorPriority(0, 0xFFFF, 0xFFFF));
}

// cpp: cascade_priority_test.cc:256-270
#[test]
fn TestGeneration() {
    let ua = CascadePriority::FromOrigin(CascadeOrigin::kUserAgent);
    let author = CascadePriority::FromOrigin(CascadeOrigin::kAuthor);

    assert_eq!(author, author);
    assert!(CascadePriority::WithAlreadyApplied(author, true) >= author);
    assert!(
        CascadePriority::WithAlreadyApplied(author, true)
            >= CascadePriority::WithAlreadyApplied(author, true)
    );
    assert_eq!(
        CascadePriority::WithAlreadyApplied(author, true),
        CascadePriority::WithAlreadyApplied(author, true)
    );

    assert!(ua < author);
    assert!(CascadePriority::WithAlreadyApplied(ua, true) < author);
    assert!(
        CascadePriority::WithAlreadyApplied(ua, true)
            < CascadePriority::WithAlreadyApplied(author, true)
    );
    assert!(
        CascadePriority::WithAlreadyApplied(ua, true)
            < CascadePriority::WithAlreadyApplied(author, true)
    );
    assert!(
        CascadePriority::WithAlreadyApplied(ua, true)
            < CascadePriority::WithAlreadyApplied(author, true)
    );
}

// cpp: cascade_priority_test.cc:272-284
#[test]
fn TestAlreadyAppliedOverwrite() {
    let mut ua = CascadePriority::FromOrigin(CascadeOrigin::kUserAgent);

    for aa in [false, true] {
        ua = CascadePriority::WithAlreadyApplied(ua, aa);
        assert_eq!(aa, ua.IsAlreadyApplied());
    }

    for aa in [true, false] {
        ua = CascadePriority::WithAlreadyApplied(ua, aa);
        assert_eq!(aa, ua.IsAlreadyApplied());
    }
}

// cpp: cascade_priority_test.cc:286-304
#[test]
fn TestPositionEncoding() {
    let mut pos: u16 = 0;
    loop {
        assert_eq!(pos as usize, AuthorPriority(0, pos, 0).GetRuleIndex());
        assert_eq!(
            pos as usize,
            AuthorPriority(0, 0, pos).GetDeclarationIndex()
        );
        pos = (pos << 1) | 1;
        if pos == u16::MAX {
            break;
        }
    }
    pos = 1;
    loop {
        assert_eq!(pos as usize, AuthorPriority(0, pos, 0).GetRuleIndex());
        assert_eq!(
            pos as usize,
            AuthorPriority(0, 0, pos).GetDeclarationIndex()
        );
        pos <<= 1;
        if pos == 0 {
            break;
        }
    }
}

// cpp: cascade_priority_test.cc:306-318
#[test]
fn TestEncodeLayerOrder() {
    assert_eq!(0, EncodeLayerOrder(0, false));
    assert_eq!(1, EncodeLayerOrder(1, false));
    assert_eq!(2, EncodeLayerOrder(2, false));
    assert_eq!(100, EncodeLayerOrder(100, false));
    assert_eq!(0xFFFF, EncodeLayerOrder(0xFFFF, false));

    assert_eq!(0 ^ 0xFFFF, EncodeLayerOrder(0, true));
    assert_eq!(1 ^ 0xFFFF, EncodeLayerOrder(1, true));
    assert_eq!(2 ^ 0xFFFF, EncodeLayerOrder(2, true));
    assert_eq!(100 ^ 0xFFFF, EncodeLayerOrder(100, true));
    assert_eq!(0xFFFF ^ 0xFFFF, EncodeLayerOrder(0xFFFF, true));
}

// cpp: cascade_priority_test.cc:320-325
#[test]
fn TestLayerOrder() {
    assert!(
        Priority(Options {
            layer_order: 1,
            ..Options::default()
        }) >= Priority(Options {
            layer_order: 0,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            layer_order: 7,
            ..Options::default()
        }) >= Priority(Options {
            layer_order: 6,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            layer_order: 42,
            ..Options::default()
        }) >= Priority(Options {
            layer_order: 42,
            ..Options::default()
        })
    );
    assert!(
        !(Priority(Options {
            layer_order: 1,
            ..Options::default()
        }) >= Priority(Options {
            layer_order: 8,
            ..Options::default()
        }))
    );
}

// cpp: cascade_priority_test.cc:327-336
#[test]
fn TestLayerOrderImportant() {
    assert!(
        Priority(Options {
            important: true,
            layer_order: 0,
            ..Options::default()
        }) >= Priority(Options {
            important: true,
            layer_order: 1,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            important: true,
            layer_order: 6,
            ..Options::default()
        }) >= Priority(Options {
            important: true,
            layer_order: 7,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            important: true,
            layer_order: 4,
            ..Options::default()
        }) >= Priority(Options {
            important: true,
            layer_order: 4,
            ..Options::default()
        })
    );
    assert!(
        !(Priority(Options {
            important: true,
            layer_order: 8,
            ..Options::default()
        }) >= Priority(Options {
            important: true,
            layer_order: 1,
            ..Options::default()
        }))
    );
}

// cpp: cascade_priority_test.cc:338-345
#[test]
fn TestLayerOrderDifferentOrigin() {
    // Layer order does not matter if the origin is different.
    let transition = CascadeOrigin::kTransition;
    assert!(
        Priority(Options {
            origin: transition,
            layer_order: 1,
            ..Options::default()
        }) >= Priority(Options {
            layer_order: 42,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            origin: transition,
            layer_order: 1,
            ..Options::default()
        }) >= Priority(Options {
            layer_order: 1,
            ..Options::default()
        })
    );
}

// cpp: cascade_priority_test.cc:347-376
#[test]
fn TestInlineStyle() {
    let user = CascadeOrigin::kUser;

    // Non-important inline style priorities
    assert!(
        Priority(Options {
            is_inline_style: true,
            ..Options::default()
        }) >= Priority(Options {
            declaration_index: 1,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            is_inline_style: true,
            ..Options::default()
        }) >= Priority(Options {
            layer_order: 1,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            tree_order: 1,
            is_inline_style: true,
            ..Options::default()
        }) >= Priority(Options {
            is_inline_style: false,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            tree_order: 1,
            is_inline_style: true,
            ..Options::default()
        }) < Priority(Options {
            tree_order: 2,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            is_inline_style: true,
            ..Options::default()
        }) >= Priority(Options {
            origin: user,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            is_inline_style: true,
            ..Options::default()
        }) < Priority(Options {
            important: true,
            ..Options::default()
        })
    );

    // Important inline style priorities
    assert!(
        Priority(Options {
            important: true,
            is_inline_style: true,
            ..Options::default()
        }) >= Priority(Options {
            important: true,
            declaration_index: 1,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            important: true,
            is_inline_style: true,
            ..Options::default()
        }) >= Priority(Options {
            important: true,
            layer_order: 1,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            important: true,
            tree_order: 1,
            is_inline_style: true,
            ..Options::default()
        }) < Priority(Options {
            important: true,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            important: true,
            tree_order: 1,
            is_inline_style: true,
            ..Options::default()
        }) >= Priority(Options {
            important: true,
            tree_order: 2,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            important: true,
            is_inline_style: true,
            ..Options::default()
        }) < Priority(Options {
            origin: user,
            important: true,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            important: true,
            is_inline_style: true,
            ..Options::default()
        }) >= Priority(Options {
            is_inline_style: false,
            ..Options::default()
        })
    );
}

// cpp: cascade_priority_test.cc:378-397
#[test]
fn TestTryStyle() {
    assert!(
        Priority(Options {
            is_try_style: true,
            ..Options::default()
        }) >= Priority(Options {
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            is_try_style: true,
            ..Options::default()
        }) >= Priority(Options {
            is_inline_style: true,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            is_try_style: true,
            ..Options::default()
        }) >= Priority(Options {
            layer_order: (EncodeLayerOrder(1, false) as u16),
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            is_try_style: true,
            ..Options::default()
        }) >= Priority(Options {
            declaration_index: 1000,
            ..Options::default()
        })
    );

    assert!(
        Priority(Options {
            is_try_style: true,
            ..Options::default()
        }) < Priority(Options {
            important: true,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            is_try_style: true,
            ..Options::default()
        }) < Priority(Options {
            origin: CascadeOrigin::kAnimation,
            ..Options::default()
        })
    );
    assert!(
        Priority(Options {
            is_try_style: true,
            ..Options::default()
        }) < Priority(Options {
            origin: CascadeOrigin::kTransition,
            ..Options::default()
        })
    );

    // Try styles generate a separate layer.
    assert_ne!(
        Priority(Options {
            is_try_style: true,
            ..Options::default()
        })
        .ForLayerComparison(),
        Priority(Options {
            ..Options::default()
        })
        .ForLayerComparison()
    );
}

// cpp: cascade_priority_test.cc:399-426
#[test]
fn TestTryTacticsStyle() {
    // Should be stronger than try-style.
    assert!(
        Priority(Options {
            is_try_tactics_style: true,
            ..Options::default()
        }) >= Priority(Options {
            is_try_style: true,
            ..Options::default()
        })
    );

    // Should be stronger than inline styles.
    assert!(
        Priority(Options {
            is_try_tactics_style: true,
            ..Options::default()
        }) >= Priority(Options {
            is_inline_style: true,
            ..Options::default()
        })
    );

    // Should be stronger than author cascade layers.
    assert!(
        Priority(Options {
            is_try_tactics_style: true,
            ..Options::default()
        }) >= Priority(Options {
            layer_order: 1000,
            ..Options::default()
        })
    );

    // Should be weaker than important in the same origin
    assert!(
        Priority(Options {
            is_try_tactics_style: true,
            ..Options::default()
        }) < Priority(Options {
            important: true,
            ..Options::default()
        })
    );

    // Should be weaker than a stronger origin.
    assert!(
        Priority(Options {
            is_try_tactics_style: true,
            ..Options::default()
        }) < Priority(Options {
            origin: CascadeOrigin::kTransition,
            ..Options::default()
        })
    );

    // Try-tactics styles generate a separate layer.
    assert_ne!(
        Priority(Options {
            is_try_tactics_style: true,
            ..Options::default()
        })
        .ForLayerComparison(),
        Priority(Options {
            ..Options::default()
        })
        .ForLayerComparison()
    );
    // Also a separate layer vs. the try styles.
    assert_ne!(
        Priority(Options {
            is_try_tactics_style: true,
            ..Options::default()
        })
        .ForLayerComparison(),
        Priority(Options {
            is_try_style: true,
            ..Options::default()
        })
        .ForLayerComparison()
    );
}

// cpp: cascade_priority_test.cc:428-525
#[test]
fn TestForLayerComparison() {
    let user = CascadeOrigin::kUser;

    assert_eq!(
        Priority(Options {
            layer_order: 1,
            declaration_index: 2,
            ..Options::default()
        })
        .ForLayerComparison(),
        Priority(Options {
            layer_order: 1,
            declaration_index: 8,
            ..Options::default()
        })
        .ForLayerComparison()
    );
    assert_eq!(
        Priority(Options {
            important: true,
            tree_order: 1,
            layer_order: 1,
            declaration_index: 4,
            ..Options::default()
        })
        .ForLayerComparison(),
        Priority(Options {
            important: true,
            tree_order: 1,
            layer_order: 1,
            declaration_index: 8,
            ..Options::default()
        })
        .ForLayerComparison()
    );
    assert_eq!(
        Priority(Options {
            important: true,
            tree_order: 1,
            layer_order: 1,
            declaration_index: 16,
            ..Options::default()
        })
        .ForLayerComparison(),
        Priority(Options {
            tree_order: 1,
            layer_order: 1,
            declaration_index: 32,
            ..Options::default()
        })
        .ForLayerComparison()
    );
    assert_eq!(
        Priority(Options {
            important: true,
            tree_order: 1,
            is_inline_style: true,
            declaration_index: 16,
            ..Options::default()
        })
        .ForLayerComparison(),
        Priority(Options {
            tree_order: 1,
            is_inline_style: true,
            declaration_index: 32,
            ..Options::default()
        })
        .ForLayerComparison()
    );

    assert!(
        Priority(Options {
            origin: user,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                origin: user,
                layer_order: 1,
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                origin: user,
                is_inline_style: true,
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                origin: user,
                tree_order: 1,
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            important: true,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                origin: user,
                layer_order: 1,
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            important: true,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                origin: user,
                is_inline_style: true,
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            important: true,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                origin: user,
                tree_order: 1,
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            important: true,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                origin: user,
                important: true,
                layer_order: 1,
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                origin: user,
                important: true,
                is_inline_style: true,
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                origin: user,
                important: true,
                tree_order: 1,
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                important: true,
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            important: true,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                origin: user,
                important: true,
                layer_order: 1,
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            important: true,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                origin: user,
                important: true,
                is_inline_style: true,
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            important: true,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                origin: user,
                important: true,
                tree_order: 1,
                ..Options::default()
            })
            .ForLayerComparison()
    );
    assert!(
        Priority(Options {
            origin: user,
            important: true,
            declaration_index: 1,
            ..Options::default()
        })
        .ForLayerComparison()
            < Priority(Options {
                important: true,
                ..Options::default()
            })
            .ForLayerComparison()
    );
}
