use foundation::String;

// cpp: layoutng_style/style/text_overflow_data.h:18
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum TextOverflowType {
    kClip,
    kEllipsis,
    kString,
}

// cpp: layoutng_style/style/text_overflow_data.h:13-41
#[derive(Clone, Debug, PartialEq)]
pub struct TextOverflowData {
    type_: TextOverflowType,
    string_value_: String,
}

#[allow(non_snake_case)]
impl TextOverflowData {
    // cpp: layoutng_style/style/text_overflow_data.h:20-23
    pub fn from_type(type_: TextOverflowType) -> Self {
        debug_assert_ne!(type_, TextOverflowType::kString);
        Self {
            type_,
            string_value_: String::from(""),
        }
    }

    // cpp: layoutng_style/style/text_overflow_data.h:24-25
    pub fn from_string(string_value: String) -> Self {
        Self {
            type_: TextOverflowType::kString,
            string_value_: string_value,
        }
    }

    // cpp: layoutng_style/style/text_overflow_data.h:31-33
    pub fn IsClip(&self) -> bool {
        self.type_ == TextOverflowType::kClip
    }
    pub fn IsEllipsis(&self) -> bool {
        self.type_ == TextOverflowType::kEllipsis
    }
    pub fn IsString(&self) -> bool {
        self.type_ == TextOverflowType::kString
    }

    // cpp: layoutng_style/style/text_overflow_data.h:35-36
    pub fn GetType(&self) -> TextOverflowType {
        self.type_
    }
    pub fn StringValue(&self) -> &String {
        &self.string_value_
    }
}
