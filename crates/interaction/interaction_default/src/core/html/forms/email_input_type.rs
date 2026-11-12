/*
 * This file is part of the WebKit project.
 *
 * Copyright (C) 2009 Michelangelo De Simone <micdesim@gmail.com>
 * Copyright (C) 2010 Google Inc. All rights reserved.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Library General Public
 * License as published by the Free Software Foundation; either
 * version 2 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Library General Public License for more details.
 *
 * You should have received a copy of the GNU Library General Public License
 * along with this library; see the file COPYING.LIB.  If not, write to
 * the Free Software Foundation, Inc., 51 Franklin Street, Fifth Floor,
 * Boston, MA 02110-1301, USA.
 *
 */

// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: email_input_type.h:42-97; email_input_type.cc:55-430.
use super::{
    base_text_input_type::{BaseTextInputType, ScriptRegexp, ScriptRegexpFactory},
    input_type::Type,
    text_field_input_type::{TextFieldInputType, TextString},
    url_input_type::StripLeadingAndTrailingHtmlSpaces,
};
use std::{cell::Cell, rc::Rc};
const kLocalPartCharacters: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789!#$%&'*+/=?^_`{|}~.-";
pub const kEmailPattern: &str = "[a-z0-9!#$%&'*+/=?^_`{|}~.-]+@[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)*";
/// Result from ICU IDNA::nameToASCII, including both U_FAILURE and IDNAInfo.
pub struct EmailIdnaResult {
    pub domain: TextString,
    pub failed: bool,
    pub has_errors: bool,
}
pub trait EmailRegexpFactory {
    fn CreateAsciiCaseInsensitive(&self, pattern: &str) -> Rc<dyn ScriptRegexp>;
}
pub trait EmailAddressServices {
    // Document::EnsureEmailRegexp caches this exact ASCII-insensitive pattern.
    fn EnsureEmailRegexp(&self, pattern: &str) -> Rc<dyn ScriptRegexp>;
    fn NameToASCII(&self, host: &[u16], options: u32) -> EmailIdnaResult;
    fn ConvertIDNToUnicode(&self, host: &[u16]) -> TextString;
}
#[derive(Clone, Copy)]
pub enum EmailValidationMessage {
    Empty,
    NoAtSign,
    EmptyLocal,
    EmptyDomain,
    InvalidLocal,
    InvalidDomain,
    InvalidDots,
    Multiple,
    Email,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum EmailVerificationState {
    kNone,
    kLoading,
    kVerified,
    kLoggedOutOrUnsupported,
    kFailed,
}
pub struct EmailVerificationTokenFacts {
    pub is_token_field: bool,
    pub nonce_is_empty: bool,
}
pub trait EmailInputElement {
    fn ValidationMessage(
        &self,
        kind: EmailValidationMessage,
        first: &[u16],
        second: &[u16],
    ) -> TextString;
    fn EmailVerificationStatusIndicatorEnabled(&self) -> bool;
    fn TextFieldCreateShadowSubtree(&self);
    fn AppendEmailVerificationIndicator(&self);
    fn HasEmailVerificationIndicator(&self) -> bool;
    fn SetEmailVerificationIndicatorState(&self, state: &'static str);
    fn IsInCanvasSubtree(&self) -> bool;
    fn FormListedInputCount(&self) -> Option<usize>;
    fn FormListedInputTokenFacts(&self, index: usize) -> Option<EmailVerificationTokenFacts>;
}
pub struct EmailInputType {
    pub base_text: BaseTextInputType,
    addresses: Rc<dyn EmailAddressServices>,
    element: Rc<dyn EmailInputElement>,
    email_verification_state_: Cell<EmailVerificationState>,
}
impl EmailInputType {
    pub fn AllowFrom(input_type: &super::input_type::InputType) -> bool {
        input_type.r#type() == Type::kEmail
    }
    pub fn new(
        mut field: TextFieldInputType,
        regexp: Rc<dyn ScriptRegexpFactory>,
        addresses: Rc<dyn EmailAddressServices>,
        element: Rc<dyn EmailInputElement>,
    ) -> Self {
        field.input_type = Type::kEmail;
        Self {
            base_text: BaseTextInputType::new(field, regexp),
            addresses,
            element,
            email_verification_state_: Cell::new(EmailVerificationState::kNone),
        }
    }
    pub fn CountUsage(
        &self,
        counter: &dyn super::input_type::InputTypeUsageCounter,
        has_maxlength_attribute: bool,
    ) {
        use super::input_type::InputTypeUsageFeature as F;
        counter.CountUsageIfVisible(F::Email);
        if has_maxlength_attribute {
            counter.CountUsageIfVisible(F::EmailMaxLength);
        }
        if self.base_text.text_field.value.Multiple() {
            counter.CountUsageIfVisible(F::EmailMultiple);
            if has_maxlength_attribute {
                counter.CountUsageIfVisible(F::EmailMultipleMaxLength);
            }
        }
    }
    pub fn CreateEmailRegexp(factory: &dyn EmailRegexpFactory) -> Rc<dyn ScriptRegexp> {
        factory.CreateAsciiCaseInsensitive(kEmailPattern)
    }
    // Source: cc:80-83. StringView::Split preserves empty addresses.
    pub fn ParseMultipleValues(value: &[u16]) -> impl Iterator<Item = &[u16]> {
        value.split(|&ch| ch == b',' as u16)
    }
    pub fn IsValidEmailAddress(regexp: &dyn ScriptRegexp, address: &[u16]) -> bool {
        if address.is_empty() {
            return false;
        }
        let (offset, length) = regexp.Match(address, 0);
        offset == 0 && length == address.len()
    }
    // Source: cc:85-121. Only ICU's source options/results cross this boundary.
    pub fn ConvertEmailAddressToAscii(
        &self,
        regexp: &dyn ScriptRegexp,
        address: &[u16],
    ) -> TextString {
        if address.iter().all(|&ch| ch <= 0x7f) {
            return address.to_vec();
        }
        let Some(at) = address.iter().position(|&ch| ch == b'@' as u16) else {
            return address.to_vec();
        };
        let converted = self
            .addresses
            .NameToASCII(&address[at + 1..], 4 /* UIDNA_CHECK_BIDI */);
        if converted.failed || converted.has_errors || converted.domain.len() > 255 {
            return address.to_vec();
        }
        let mut ascii = address[..=at].to_vec();
        ascii.extend(converted.domain);
        if Self::IsValidEmailAddress(regexp, &ascii) {
            ascii
        } else {
            address.to_vec()
        }
    }
    pub fn ConvertEmailAddressToUnicode(&self, address: &[u16]) -> TextString {
        if !address.iter().all(|&ch| ch <= 0x7f) {
            return address.to_vec();
        }
        let Some(at) = address.iter().position(|&ch| ch == b'@' as u16) else {
            return address.to_vec();
        };
        if !address[at + 1..]
            .windows(4)
            .any(|chars| chars == [b'x' as u16, b'n' as u16, b'-' as u16, b'-' as u16])
        {
            return address.to_vec();
        }
        let mut unicode = address[..=at].to_vec();
        unicode.extend(self.addresses.ConvertIDNToUnicode(&address[at + 1..]));
        unicode
    }
    // Source: cc:199-216. None is the source null String (valid), while
    // Some(empty) identifies an invalid empty item in a multiple-address list.
    pub fn FindInvalidAddress(&self, value: &[u16]) -> Option<TextString> {
        let regexp = self.addresses.EnsureEmailRegexp(kEmailPattern);
        Self::FindInvalidAddressWithRegexp(
            regexp.as_ref(),
            value,
            self.base_text.text_field.value.Multiple(),
        )
    }
    pub fn FindInvalidAddressWithRegexp(
        regexp: &dyn ScriptRegexp,
        value: &[u16],
        multiple: bool,
    ) -> Option<TextString> {
        if value.is_empty() {
            return None;
        }
        if !multiple {
            return (!Self::IsValidEmailAddress(regexp, value)).then(|| value.to_vec());
        }
        for address in Self::ParseMultipleValues(value) {
            let address = StripLeadingAndTrailingHtmlSpaces(address);
            if !Self::IsValidEmailAddress(regexp, address) {
                return Some(address.to_vec());
            }
        }
        None
    }
    pub fn TypeMismatchFor(&self, value: &[u16]) -> bool {
        self.FindInvalidAddress(value).is_some()
    }
    pub fn TypeMismatch(&self) -> bool {
        self.TypeMismatchFor(&self.base_text.text_field.value.Value())
    }
    // Source: cc:226-281. Locale messages use the same offending UTF-16 span,
    // including a surrogate pair selected at the source invalid-character site.
    pub fn TypeMismatchText(&self) -> TextString {
        use EmailValidationMessage as M;
        let invalid = self
            .FindInvalidAddress(&self.base_text.text_field.value.Value())
            .expect("type mismatch address");
        if invalid.is_empty() {
            return self.element.ValidationMessage(M::Empty, &[], &[]);
        }
        let at_sign = [b'@' as u16];
        let Some(at) = invalid.iter().position(|&ch| ch == b'@' as u16) else {
            return self
                .element
                .ValidationMessage(M::NoAtSign, &at_sign, &invalid);
        };
        let unicode = self.ConvertEmailAddressToUnicode(&invalid);
        let local = &invalid[..at];
        let domain = &invalid[at + 1..];
        if local.is_empty() {
            return self
                .element
                .ValidationMessage(M::EmptyLocal, &at_sign, &unicode);
        }
        if domain.is_empty() {
            return self
                .element
                .ValidationMessage(M::EmptyDomain, &at_sign, &unicode);
        }
        if let Some(index) = local.iter().position(|&ch| IsInvalidLocalPartCharacter(ch)) {
            let len = if (0xd800..=0xdbff).contains(&local[index]) {
                2
            } else {
                1
            };
            return self.element.ValidationMessage(
                M::InvalidLocal,
                &at_sign,
                &local[index..(index + len).min(local.len())],
            );
        }
        if let Some(index) = domain.iter().position(|&ch| IsInvalidDomainCharacter(ch)) {
            let len = if (0xd800..=0xdbff).contains(&domain[index]) {
                2
            } else {
                1
            };
            return self.element.ValidationMessage(
                M::InvalidDomain,
                &at_sign,
                &domain[index..(index + len).min(domain.len())],
            );
        }
        if !CheckValidDotUsage(domain) {
            let unicode_at = unicode
                .iter()
                .position(|&ch| ch == b'@' as u16)
                .expect("unicode email retains at sign");
            return self.element.ValidationMessage(
                M::InvalidDots,
                &[b'.' as u16],
                &unicode[unicode_at + 1..],
            );
        }
        self.element.ValidationMessage(
            if self.base_text.text_field.value.Multiple() {
                M::Multiple
            } else {
                M::Email
            },
            &[],
            &[],
        )
    }
    pub fn SupportsSelectionAPI(&self) -> bool {
        false
    }
    // Source: cc:287-297. No general TextField length clipping is introduced.
    pub fn SanitizeValue(&self, proposed: &[u16]) -> TextString {
        Self::SanitizeValueWithMultiple(proposed, self.base_text.text_field.value.Multiple())
    }
    pub fn SanitizeValueWithMultiple(proposed: &[u16], multiple: bool) -> TextString {
        let no_breaks: TextString = proposed
            .iter()
            .copied()
            .filter(|&ch| ch != 10 && ch != 13)
            .collect();
        if !multiple {
            return StripLeadingAndTrailingHtmlSpaces(&no_breaks).to_vec();
        }
        JoinAddresses(
            Self::ParseMultipleValues(&no_breaks)
                .map(|address| StripLeadingAndTrailingHtmlSpaces(address).to_vec()),
        )
    }
    pub fn ConvertFromVisibleValue(&self, visible: &[u16]) -> TextString {
        let sanitized = self.SanitizeValue(visible);
        let regexp = self.addresses.EnsureEmailRegexp(kEmailPattern);
        if !self.base_text.text_field.value.Multiple() {
            return self.ConvertEmailAddressToAscii(regexp.as_ref(), &sanitized);
        }
        JoinAddresses(
            Self::ParseMultipleValues(&sanitized)
                .map(|address| self.ConvertEmailAddressToAscii(regexp.as_ref(), address)),
        )
    }
    pub fn VisibleValue(&self) -> TextString {
        let value = self.base_text.text_field.value.Value();
        if !self.base_text.text_field.value.Multiple() {
            return self.ConvertEmailAddressToUnicode(&value);
        }
        JoinAddresses(
            Self::ParseMultipleValues(&value)
                .map(|address| self.ConvertEmailAddressToUnicode(address)),
        )
    }
    pub fn MultipleAttributeChanged(&self) {
        self.base_text
            .text_field
            .effects
            .SetValueFromRenderer(&self.SanitizeValue(&self.base_text.text_field.value.Value()));
    }
    // Source: cc:334-430. Feature flags and live form/shadow facts are read at
    // each call; neither the class nor the binding retains a form-tree snapshot.
    pub fn CreateShadowSubtree(&self) {
        self.element.TextFieldCreateShadowSubtree();
        if self.IsEmailVerificationStatusIndicatorEnabled() {
            self.element.AppendEmailVerificationIndicator();
            self.UpdateEmailVerificationIndicator();
        }
    }
    pub fn NeedsContainer(&self) -> bool {
        self.IsEmailVerificationStatusIndicatorEnabled()
    }
    pub fn SetEmailVerificationState(&self, state: EmailVerificationState) {
        if !self.IsEmailVerificationStatusIndicatorEnabled()
            || self.email_verification_state_.get() == state
        {
            return;
        }
        self.email_verification_state_.set(state);
        self.UpdateEmailVerificationIndicator();
    }
    pub fn GetEmailVerificationState(&self) -> EmailVerificationState {
        self.email_verification_state_.get()
    }
    pub fn UpdateEmailVerificationIndicator(&self) {
        if !self.IsEmailVerificationStatusIndicatorEnabled()
            || !self.element.HasEmailVerificationIndicator()
        {
            return;
        }
        if self.element.IsInCanvasSubtree() {
            self.element.SetEmailVerificationIndicatorState("none");
            return;
        }
        let state = match self.email_verification_state_.get() {
            EmailVerificationState::kNone => {
                if self.IsEmailVerificationSupported() {
                    "supported"
                } else {
                    "none"
                }
            }
            EmailVerificationState::kLoading => "loading",
            EmailVerificationState::kVerified => "verified",
            EmailVerificationState::kLoggedOutOrUnsupported => "logged-out-or-unsupported",
            EmailVerificationState::kFailed => "failed",
        };
        self.element.SetEmailVerificationIndicatorState(state);
    }
    pub fn IsEmailVerificationStatusIndicatorEnabled(&self) -> bool {
        self.element.EmailVerificationStatusIndicatorEnabled()
    }
    pub fn IsEmailVerificationSupported(&self) -> bool {
        let Some(length) = self.element.FormListedInputCount() else {
            return false;
        };
        for index in 0..length {
            if self
                .element
                .FormListedInputTokenFacts(index)
                .is_some_and(|input| input.is_token_field && !input.nonce_is_empty)
            {
                return true;
            }
        }
        false
    }
}
fn IsInvalidLocalPartCharacter(ch: u16) -> bool {
    ch > 0x7f || !kLocalPartCharacters.contains(&(ch as u8).to_ascii_lowercase())
}
fn IsInvalidDomainCharacter(ch: u16) -> bool {
    ch > 0x7f || (!(ch as u8).is_ascii_alphanumeric() && ch != b'.' as u16 && ch != b'-' as u16)
}
fn CheckValidDotUsage(domain: &[u16]) -> bool {
    domain.is_empty()
        || (domain.first() != Some(&(b'.' as u16))
            && domain.last() != Some(&(b'.' as u16))
            && !domain
                .windows(2)
                .any(|chars| chars == [b'.' as u16, b'.' as u16]))
}
fn JoinAddresses(addresses: impl Iterator<Item = TextString>) -> TextString {
    let mut result = Vec::new();
    for (index, address) in addresses.enumerate() {
        if index != 0 {
            result.push(b',' as u16);
        }
        result.extend(address);
    }
    result
}
