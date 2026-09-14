use super::runtime::{
    AnimaData, AnimaState, Bone, BoneClipData, BoneData, ClipData, Color, Timeline, Transform,
};

pub fn armature() -> AnimaState {
    AnimaState {
        bones: vec![
            Bone {
                transform: Transform {
                    translate: [100.0, 30.0],
                    ..Default::default()
                },
                length: 50.0,
                color: Color(0x00EECC99),
                ..Default::default()
            },
            Bone {
                transform: Transform {
                    translate: [50.0, 0.0],
                    ..Default::default()
                },
                length: 50.0,
                color: Color(0xEE00CC99),
                parent: 0,
            },
            Bone {
                transform: Transform {
                    translate: [50.0, 0.0],
                    ..Default::default()
                },
                length: 50.0,
                color: Color(0x00EECC99),
                parent: 1,
            },
        ],
        ..Default::default()
    }
}

pub fn data() -> AnimaData {
    let bone0 = "bone0";
    let bone1 = "bone1";
    let bone2 = "bone2";

    let bones = vec![
        BoneData {
            name: String::from(bone0),
            translate: [100.0, 30.0],
            length: 50.0,
            color: Color(0x00EECC99),
            ..Default::default()
        },
        BoneData {
            name: String::from(bone1),
            parent: String::from(bone0),
            translate: [50.0, 0.0],
            length: 50.0,
            color: Color(0xEE00CC99),
            ..Default::default()
        },
        BoneData {
            name: String::from(bone2),
            parent: String::from(bone1),
            translate: [50.0, 0.0],
            length: 50.0,
            color: Color(0x00EECC99),
            ..Default::default()
        },
    ];

    let clips = {
        let mut rotate = Timeline::default();
        rotate.add_linear(0, 0.0);
        rotate.add_linear(5, std::f32::consts::FRAC_PI_2);
        rotate.add_linear(8, std::f32::consts::PI);

        let bones = vec![
            BoneClipData {
                bone: String::from(bone0),
                rotate,
                ..Default::default()
            },
            BoneClipData {
                bone: String::from(bone1),
                ..Default::default()
            },
            BoneClipData {
                bone: String::from(bone2),
                ..Default::default()
            },
        ];

        let name = String::from("some animation");

        vec![ClipData { name, bones }]
    };

    AnimaData {
        bones,
        clips,
        ..Default::default()
    }
}
