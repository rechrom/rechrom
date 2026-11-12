//! Chromium image_input_type.cc:58-71. Only the activated submitter contributes
//! coordinates, including the nameless x/y branch.
use super::input_type::{FormDataAppendAdmission, InputFormDataOperations};
use interaction_dispatch::core::fileapi::form_data::FormData;
pub struct ImageInputType;
impl ImageInputType {
    pub fn AppendWithOperations(
        activated: bool,
        element: &dyn InputFormDataOperations,
        data: &mut FormData,
    ) -> Result<(), FormDataAppendAdmission> {
        if !activated {
            return Ok(());
        }
        let (x, y) = element
            .ImageCoordinatesForFormData()
            .ok_or(FormDataAppendAdmission::ImageCoordinates)?;
        let name = element.NameForFormData();
        for (suffix, value) in [(b'x', x), (b'y', y)] {
            let mut key = name.to_vec();
            if !key.is_empty() {
                key.push(u16::from(b'.'));
            }
            key.push(u16::from(suffix));
            data.AppendFromElement(&key, &value.to_string().encode_utf16().collect::<Vec<_>>());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    struct Image;
    impl InputFormDataOperations for Image {
        fn NameForFormData(&self) -> Rc<[u16]> {
            Rc::from([])
        }
        fn ValueForFormData(&self) -> Rc<[u16]> {
            Rc::from([])
        }
        fn ImageCoordinatesForFormData(&self) -> Option<(i32, i32)> {
            Some((7, 11))
        }
    }
    #[test]
    fn only_activated_image_appends_coordinates_even_without_name() {
        let mut data = FormData::default();
        ImageInputType::AppendWithOperations(false, &Image, &mut data).unwrap();
        assert!(data.Entries().is_empty());
        ImageInputType::AppendWithOperations(true, &Image, &mut data).unwrap();
        assert_eq!(data.EncodeFormData(false), b"x=7&y=11");
    }
}
