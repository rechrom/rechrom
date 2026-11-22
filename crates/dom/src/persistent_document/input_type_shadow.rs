//! Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! Source: color_input_type.cc:225-239,378-396;
//! file_input_type.cc:381-441,640-644.
//! Source-owned mutations reuse the canonical DOM shadow node identities.
use super::*;
impl PersistentDocument {
    /// EmailVerificationIndicatorElement ctor, text_control_inner_elements.cc:301-307.
    pub fn AppendEmailVerificationIndicator(&mut self, input: usize) -> usize {
        let container = self
            .TextControlContainerElement(input)
            .expect("email source requires decoration container");
        let indicator = self.CreateElement(
            DOMNamespace::kHTML,
            "div".into(),
            vec![DOMAttribute {
                local_name: "id".into(),
                value: "email-verification-indicator".into(),
                ..Default::default()
            }],
        );
        self.email_verification_indicator_hosts
            .insert(indicator, input);
        self.SetUAShadowPseudoElementIdentity(
            input,
            indicator,
            "-internal-email-verification-indicator",
        );
        self.AppendChild(container, indicator);
        indicator
    }
    pub fn EmailVerificationIndicatorHost(&self, node: usize) -> Option<usize> {
        self.email_verification_indicator_hosts.get(&node).copied()
    }
    pub fn EmailVerificationIndicator(&self, input: usize) -> Option<usize> {
        self.UAShadowPseudoElement(input, "-internal-email-verification-indicator")
            .filter(|&node| self.ContainingShadowRoot(node) == self.ShadowRoot(input))
    }

    /// TextControlElement::DirectionForFormData, cc:1281-1306.
    pub fn TextControlDirectionForFormData(&self, control: usize) -> Rc<[u16]> {
        let telephone = self.InputType(control) == Some(DOMInputType::kTelephone);
        let mut current = Some(control);
        while let Some(element) = current {
            let dir = self.arena[element]
                .attributes
                .iter()
                .find(|a| a.namespace_uri.is_empty() && a.local_name == "dir");
            if let Some(dir) = dir {
                if dir.value.eq_ignore_ascii_case("rtl") || dir.value.eq_ignore_ascii_case("ltr") {
                    return if dir.value.eq_ignore_ascii_case("rtl") {
                        Rc::from([114, 116, 108])
                    } else {
                        Rc::from([108, 116, 114])
                    };
                }
                if dir.value.eq_ignore_ascii_case("auto") {
                    return if self.CachedDirectionality(element) == foundation::TextDirection::kRtl
                    {
                        Rc::from([114, 116, 108])
                    } else {
                        Rc::from([108, 116, 114])
                    };
                }
            } else if telephone {
                break;
            }
            current = self.arena[element].parent;
            while current.is_some_and(|node| {
                self.arena[node].node_type != DOMNodeType::kElement
                    || self.arena[node].namespace != DOMNamespace::kHTML
            }) {
                current = self.arena[current.unwrap()].parent;
            }
        }
        Rc::from([108, 116, 114])
    }
    pub fn SetHasCustomStyleCallbacks(&mut self, node: usize) {
        self.arena[node].has_custom_style_callbacks = true;
    }
    pub fn HasCustomStyleCallbacks(&self, node: usize) -> bool {
        self.arena[node].has_custom_style_callbacks
    }
    /// DOMTokenList::Add/Remove/UpdateWithTokenSet (cc:102-128,238-258).
    pub fn SetClassToken(&mut self, node: usize, token: &str, present: bool) {
        assert!(!token.is_empty() && !token.bytes().any(|c| matches!(c, 9 | 10 | 12 | 13 | 32)));
        let original = self.arena[node]
            .attributes
            .iter()
            .find(|a| a.namespace_uri.is_empty() && a.local_name == "class")
            .map(|a| a.value.as_str())
            .unwrap_or("");
        let mut tokens = Vec::<&str>::new();
        for item in original
            .split(|c| matches!(c, '\t' | '\n' | '\x0c' | '\r' | ' '))
            .filter(|s| !s.is_empty())
        {
            if !tokens.contains(&item) && (present || item != token) {
                tokens.push(item);
            }
        }
        if present && !tokens.contains(&token) {
            tokens.push(token);
        }
        let value = tokens.join(" ");
        self.SetAttribute(
            node,
            DOMAttribute {
                local_name: "class".into(),
                value,
                ..Default::default()
            },
        );
    }
    pub fn TextControlEditingViewport(&self, input: usize) -> Option<usize> {
        self.TextControlContainerElement(input)?;
        self.arena[*self.text_control_inner_editors.get(&input)?].parent
    }
    pub fn SearchCancelButtonElement(&self, input: usize) -> Option<usize> {
        self.search_cancel_buttons.get(&input).copied()
    }
    pub fn PasswordRevealButtonElement(&self, input: usize) -> Option<usize> {
        self.UAShadowPseudoElement(input, "-internal-reveal")
    }
    /// PasswordRevealButtonElement ctor, text_control_inner_elements.cc:264-269.
    /// PasswordInputType retains this same native child on its real UA root.
    pub fn CreatePasswordRevealButtonElement(&mut self, input: usize) -> usize {
        assert_eq!(self.InputType(input), Some(DOMInputType::kPassword));
        if let Some(button) = self.PasswordRevealButtonElement(input) {
            return button;
        }
        let button = self.CreateElement(
            DOMNamespace::kHTML,
            "div".into(),
            vec![DOMAttribute {
                local_name: "id".into(),
                value: "password-reveal".into(),
                ..Default::default()
            }],
        );
        self.SetUAShadowPseudoElementIdentity(input, button, "-internal-reveal");
        button
    }
    pub fn FormControllerDropReferencedFilePaths(&self) -> bool {
        self.form_state_drops_referenced_file_paths.get()
    }
    pub fn SetFormControllerDropReferencedFilePaths(&self, value: bool) {
        self.form_state_drops_referenced_file_paths.set(value)
    }
    pub fn SetInputFileNoFileLabel(&mut self, element: usize, label: String) {
        self.input_file_no_file_labels
            .insert(self.arena[element].id, label);
    }
    pub fn InputFileNoFileLabel(&self, element: usize) -> Option<&str> {
        self.input_file_no_file_labels
            .get(&self.arena[element].id)
            .map(String::as_str)
    }

    pub fn CreateColorInputShadowSubtree(&mut self, element: usize) {
        assert_eq!(self.InputType(element), Some(DOMInputType::kColor));
        if self.color_swatches.contains_key(&element) {
            return;
        }
        let root = self
            .ShadowRoot(element)
            .unwrap_or_else(|| self.AttachShadowRoot(element, DOMShadowRootMode::kUserAgent));
        let wrapper = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let swatch = self.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        self.SetUAShadowPseudoElementIdentity(element, wrapper, "-webkit-color-swatch-wrapper");
        self.SetUAShadowPseudoElementIdentity(element, swatch, "-webkit-color-swatch");
        self.AppendChild(wrapper, swatch);
        self.AppendChild(root, wrapper);
        self.color_swatch_wrappers.insert(element, wrapper);
        self.color_swatches.insert(element, swatch);
    }
    pub fn SetColorInputSwatchBackgroundIfPresent(&mut self, element: usize, value: &str) {
        if let Some(&swatch) = self.color_swatches.get(&element) {
            self.SetAttribute(
                swatch,
                DOMAttribute {
                    local_name: "style".into(),
                    value: format!("background-color:{value}"),
                    ..Default::default()
                },
            );
        }
    }
    pub fn CreateFileInputShadowSubtree(&mut self, element: usize, label: &str, active: bool) {
        assert_eq!(self.InputType(element), Some(DOMInputType::kFile));
        if let Some(button) = self.FileUploadButtonElement(element) {
            // Adopt an already parsed UA subtree without changing node identity.
            self.SetFileInputUploadButtonValueIfPresent(element, label);
            self.SetElementActive(button, active);
            return;
        }
        let root = self
            .ShadowRoot(element)
            .unwrap_or_else(|| self.AttachShadowRoot(element, DOMShadowRootMode::kUserAgent));
        let button = self.CreateElement(
            DOMNamespace::kHTML,
            "input".into(),
            vec![
                DOMAttribute {
                    local_name: "type".into(),
                    value: "button".into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "value".into(),
                    value: label.into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "id".into(),
                    value: "file-upload-button".into(),
                    ..Default::default()
                },
            ],
        );
        self.SetUAShadowPseudoElementIdentity(element, button, "-webkit-file-upload-button");
        self.SetElementActive(button, active);
        self.AppendChild(root, button);
        let status = self.CreateElementDefault(DOMNamespace::kHTML, "span".into());
        self.AppendChild(root, status);
        for node in [button, status] {
            self.SetAttribute(
                node,
                DOMAttribute {
                    local_name: "aria-hidden".into(),
                    value: "true".into(),
                    ..Default::default()
                },
            );
        }
        self.file_upload_buttons.insert(element, button);
        self.file_status_elements.insert(element, status);
    }
    pub fn SetFileInputUploadButtonValueIfPresent(&mut self, element: usize, label: &str) {
        if let Some(button) = self.FileUploadButtonElement(element) {
            self.SetAttribute(
                button,
                DOMAttribute {
                    local_name: "value".into(),
                    value: label.into(),
                    ..Default::default()
                },
            );
        }
    }
    pub fn SetFileInputUploadButtonDisabledIfPresent(&mut self, element: usize, disabled: bool) {
        if let Some(button) = self.FileUploadButtonElement(element) {
            if disabled {
                self.SetAttribute(
                    button,
                    DOMAttribute {
                        local_name: "disabled".into(),
                        value: String::new(),
                        ..Default::default()
                    },
                );
            } else {
                self.RemoveAttributeDefault(button, "disabled");
            }
        }
    }
    pub fn SetFileInputStatusTextIfPresent(&mut self, element: usize, text: &str) {
        if let Some(status) = self.FileStatusElement(element) {
            self.SetTextContent(status, text.into());
        }
    }
}

impl PersistentDocument {
    // Source: image_input_type.cc:131-138,189-225,247-267.
    pub fn SetImageInputShadowAltTextIfDifferent(&mut self, element: usize, value: &str) {
        let Some(root) = self.ShadowRoot(element) else {
            return;
        };
        let Some(text) = self.FindElementByIdInTreeScope(root, "alttext") else {
            return;
        };
        let units: Vec<u16> = value.encode_utf16().collect();
        if self.DescendantTextUTF16(text).as_ref() != units.as_slice() {
            self.SetTextContentUTF16(text, &units);
        }
    }
    pub fn ImageInputDimensionAttribute(&self, element: usize, width: bool) -> Option<u32> {
        self.arena[element]
            .FindAttribute(if width { "width" } else { "height" })
            .and_then(|a| Self::ParseHTMLNonNegativeInteger(&a.value))
    }
    pub fn RemoveImageInputShadowChildrenIfPresent(&mut self, element: usize) {
        if let Some(root) = self.ShadowRoot(element) {
            while let Some(child) = self.arena[root].children.first().copied() {
                self.Remove(child);
            }
        }
    }
}

impl PersistentDocument {
    // Source: HTMLInputElement::files / FileInputType constructor. Lazy DOM
    // allocation and the InputType share this same list; switching type drops
    // only the element's reference, so script-retained old lists remain valid.
    pub fn InputFiles(&self, element: usize) -> Option<Rc<fileapi::FileList>> {
        if self.InputType(element) != Some(DOMInputType::kFile) {
            return None;
        }
        Some(
            self.input_file_lists
                .borrow_mut()
                .entry(self.arena[element].id)
                .or_insert_with(|| Rc::new(fileapi::FileList::default()))
                .clone(),
        )
    }
    pub fn StoreInputFiles(&mut self, element: usize, files: Rc<fileapi::FileList>) {
        self.input_file_lists
            .borrow_mut()
            .insert(self.arena[element].id, files);
    }
    pub fn AssignInputFiles(&mut self, element: usize, files: Rc<fileapi::FileList>) {
        if self.InputType(element) != Some(DOMInputType::kFile) {
            return;
        }
        if self.attribute_lifecycle.HasOwner() {
            self.attribute_lifecycle
                .RecordInputFilesAssigned(self.arena[element].id, files);
        } else {
            self.StoreInputFiles(element, files);
            self.SetControlNeedsValidityCheck(element);
        }
    }
}
