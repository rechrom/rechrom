use foundation::{CppOstream, Length};

// cpp: layoutng_style/style/transform_origin.h:16-33
#[derive(Clone)]
pub struct TransformOrigin {
    x_: Length,
    y_: Length,
    z_: f32,
}

#[allow(non_snake_case)]
impl TransformOrigin {
    // cpp: layoutng_style/style/transform_origin.h:20-21
    pub fn new(x: &Length, y: &Length, z: f32) -> Self {
        Self {
            x_: x.clone(),
            y_: y.clone(),
            z_: z,
        }
    }

    // cpp: layoutng_style/style/transform_origin.h:25
    pub fn X(&self) -> &Length {
        &self.x_
    }

    // cpp: layoutng_style/style/transform_origin.h:26
    pub fn Y(&self) -> &Length {
        &self.y_
    }

    // cpp: layoutng_style/style/transform_origin.h:27
    pub fn Z(&self) -> f32 {
        self.z_
    }
}

// cpp: layoutng_style/style/transform_origin.h:22-24
impl PartialEq for TransformOrigin {
    fn eq(&self, other: &Self) -> bool {
        self.x_ == other.x_ && self.y_ == other.y_ && self.z_ == other.z_
    }
}

// cpp: layoutng_style/style/transform_origin.h:35-42
#[allow(non_snake_case)]
impl TransformOrigin {
    pub fn WriteToCppOstream<'a>(&self, stream: &'a mut dyn CppOstream) -> &'a mut dyn CppOstream {
        stream.WriteStr("TransformOrigin{");
        stream.WriteStr("x=");
        stream.WriteLength(self.X());
        stream.WriteStr(" y=");
        stream.WriteLength(self.Y());
        stream.WriteStr(" z=");
        stream.WriteFloat(self.Z());
        stream.WriteStr("}");
        stream
    }
}
