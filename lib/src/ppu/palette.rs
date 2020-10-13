use crate::ppu::color::Color;

#[derive(Debug)]
pub struct Palette {
    pub colors: Vec<Color>,
}

impl Default for Palette {
    fn default() -> Self {
        let colors = vec![
            Color::new(84 , 84 , 84 ),  Color::new( 0 , 30 ,116 ),  Color::new( 8 , 16 ,144 ), Color::new( 48 ,  0 ,136 ),  Color::new(68 ,  0 ,100 ),  Color::new(92 ,  0 , 48 ),  Color::new(84 ,  4 ,  0 ),  Color::new(60 , 24 ,  0 ),  Color::new(32 , 42 ,  0 ),  Color::new( 8 , 58 ,  0 ),  Color::new( 0 , 64 ,  0 ),  Color::new( 0 , 60 ,  0 ),  Color::new( 0 , 50 , 60 ), Color::new(  0 ,  0 ,  0 ),
            Color::new(152, 150, 152),  Color::new(  8,  76, 196),  Color::new( 48,  50, 236), Color::new(  92,  30, 228),  Color::new(136,  20, 176),  Color::new(160,  20, 100),  Color::new(152,  34,  32),  Color::new(120,  60,   0),  Color::new( 84,  90,   0),  Color::new( 40, 114,   0),  Color::new(  8, 124,   0),  Color::new(  0, 118,  40),  Color::new(  0, 102, 120), Color::new(   0,   0,   0),
            Color::new(236, 238, 236),  Color::new( 76, 154, 236),  Color::new(120, 124, 236), Color::new( 176,  98, 236),  Color::new(228,  84, 236),  Color::new(236,  88, 180),  Color::new(236, 106, 100),  Color::new(212, 136,  32),  Color::new(160, 170,   0),  Color::new(116, 196,   0),  Color::new( 76, 208,  32),  Color::new( 56, 204, 108),  Color::new( 56, 180, 204), Color::new(  60,  60,  60),
            Color::new(236, 238, 236),  Color::new(168, 204, 236),  Color::new(188, 188, 236), Color::new( 212, 178, 236),  Color::new(236, 174, 236),  Color::new(236, 174, 212),  Color::new(236, 180, 176),  Color::new(228, 196, 144),  Color::new(204, 210, 120),  Color::new(180, 222, 120),  Color::new(168, 226, 144),  Color::new(152, 226, 180),  Color::new(160, 214, 228), Color::new( 160, 162, 160),
        ];

        Self {
            colors
        }
    }
}

impl Palette {
    pub fn new(colors: &[Color]) -> Self {
        Self {
            colors: colors.to_vec()
        }
    }
}