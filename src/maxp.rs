use crate::{
    data_types::{uint16, Version16Dot16},
    decoder::{FromData, Stream},
};

#[allow(non_snake_case)]
pub struct MaxpTable {
    pub version: Version16Dot16, // version
    pub numGlyphs: uint16,       //The number of glyphs in the font.
    pub version1: Option<MaxpTableVersion1Extension>,
}

impl MaxpTable {
    #[allow(non_snake_case)]
    pub fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let version = s.read()?;
        let numGlyphs = s.read()?;
        let version1 = match version {
            Version16Dot16(00010000) => Some(s.read()?),
            _ => None,
        };
        Some(Self {
            version,
            numGlyphs,
            version1,
        })
    }
}

#[allow(non_snake_case)]
pub struct MaxpTableVersion1Extension {
    pub maxPoints: uint16,             //Maximum points in a non-composite glyph.
    pub maxContours: uint16,           //Maximum contours in a non-composite glyph.
    pub maxCompositePoints: uint16,    //Maximum points in a composite glyph.
    pub maxCompositeContours: uint16,  //Maximum contours in a composite glyph.
    pub maxZones: uint16, //1 if instructions do not use the twilight zone (Z0), or 2 if instructions do use Z0; should be set to 2 in most cases.
    pub maxTwilightPoints: uint16, //Maximum points used in Z0.
    pub maxStorage: uint16, //Number of Storage Area locations.
    pub maxFunctionDefs: uint16, //Number of FDEFs, equal to the highest function number + 1.
    pub maxInstructionDefs: uint16, //Number of IDEFs.
    pub maxStackElements: uint16, //Maximum stack depth across Font Program ('fpgm' table), CVT Program ('prep' table) and all glyph instructions (in the 'glyf' table).
    pub maxSizeOfInstructions: uint16, //Maximum byte count for glyph instructions.
    pub maxComponentElements: uint16, //Maximum number of components referenced at “top level” for any composite glyph.
    pub maxComponentDepth: uint16,    //Maximum levels of recursion; 1 for simple components.
}

impl FromData for MaxpTableVersion1Extension {
    const SIZE: usize = 2 * 13;
    #[allow(non_snake_case)]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let maxPoints = s.read()?;
        let maxContours = s.read()?;
        let maxCompositePoints = s.read()?;
        let maxCompositeContours = s.read()?;
        let maxZones = s.read()?;
        let maxTwilightPoints = s.read()?;
        let maxStorage = s.read()?;
        let maxFunctionDefs = s.read()?;
        let maxInstructionDefs = s.read()?;
        let maxStackElements = s.read()?;
        let maxSizeOfInstructions = s.read()?;
        let maxComponentElements = s.read()?;
        let maxComponentDepth = s.read()?;
        Some(Self {
            maxPoints,
            maxContours,
            maxCompositePoints,
            maxCompositeContours,
            maxZones,
            maxTwilightPoints,
            maxStorage,
            maxFunctionDefs,
            maxInstructionDefs,
            maxStackElements,
            maxSizeOfInstructions,
            maxComponentElements,
            maxComponentDepth,
        })
    }
}
