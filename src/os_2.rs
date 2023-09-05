use core::fmt;

use crate::{
    data_types::{int16, uint16, uint32, uint8, Tag},
    decoder::{FromData, Stream},
};

pub struct Weight(pub u16);
impl FromData for Weight {
    const SIZE: usize = u16::SIZE;
    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(Self)
    }
}

impl Weight {
    pub fn to_string(&self) -> &'static str {
        match self.0 {
            100 => "Thin",
            200 => "Extra-light (Ultra-light)",
            300 => "Light",
            400 => "Normal (Regular)",
            500 => "Medium",
            600 => "Semi-Bold (Demi-bold)",
            700 => "Bold",
            800 => "Extra-bold (Ultra-bold)",
            900 => "Black (Heavy)",
            _ => "Custom",
        }
    }
}

impl fmt::Debug for Weight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.to_string(), self.0)
    }
}

#[derive(Debug)]
#[allow(non_snake_case)]
pub struct OS2Table {
    pub version: uint16,                         //
    pub xAvgCharWidth: int16,                    //
    pub usWeightClass: Weight,                   //
    pub usWidthClass: uint16,                    //
    pub fsType: uint16,                          //
    pub ySubscriptXSize: int16,                  //
    pub ySubscriptYSize: int16,                  //
    pub ySubscriptXOffset: int16,                //
    pub ySubscriptYOffset: int16,                //
    pub ySuperscriptXSize: int16,                //
    pub ySuperscriptYSize: int16,                //
    pub ySuperscriptXOffset: int16,              //
    pub ySuperscriptYOffset: int16,              //
    pub yStrikeoutSize: int16,                   //
    pub yStrikeoutPosition: int16,               //
    pub sFamilyClass: int16,                     //
    pub panose: [uint8; 10],                     //
    pub ulUnicodeRange1: uint32,                 // Bits 0–31
    pub ulUnicodeRange2: uint32,                 // Bits 32–63
    pub ulUnicodeRange3: uint32,                 // Bits 64–95
    pub ulUnicodeRange4: uint32,                 // Bits 96–127
    pub achVendID: Tag,                          //
    pub fsSelection: uint16,                     //
    pub usFirstCharIndex: uint16,                //
    pub usLastCharIndex: uint16,                 // Apple TrueType Version0
    pub sTypoAscender: Option<int16>,            // OpenType Spec Version0+
    pub sTypoDescender: Option<int16>,           // OpenType Spec Version0+
    pub sTypoLineGap: Option<int16>,             // OpenType Spec Version0+
    pub usWinAscent: Option<uint16>,             // OpenType Spec Version0+
    pub usWinDescent: Option<uint16>,            // OpenType Spec Version0+
    pub ulCodePageRange1: Option<uint32>,        // Version1+
    pub ulCodePageRange2: Option<uint32>,        // Version1+
    pub sxHeight: Option<int16>,                 // Version4+
    pub sCapHeight: Option<int16>,               // Version4+
    pub usDefaultChar: Option<uint16>,           // Version4+
    pub usBreakChar: Option<uint16>,             // Version4+
    pub usMaxContext: Option<uint16>,            // Version4+
    pub usLowerOpticalPointSize: Option<uint16>, // Version5+
    pub usUpperOpticalPointSize: Option<uint16>, // Version5+
}

impl OS2Table {
    #[allow(non_snake_case)]
    pub fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let version = s.read()?;
        let xAvgCharWidth = s.read()?;
        let usWeightClass = s.read()?;
        let usWidthClass = s.read()?;
        let fsType = s.read()?;
        let ySubscriptXSize = s.read()?;
        let ySubscriptYSize = s.read()?;
        let ySubscriptXOffset = s.read()?;
        let ySubscriptYOffset = s.read()?;
        let ySuperscriptXSize = s.read()?;
        let ySuperscriptYSize = s.read()?;
        let ySuperscriptXOffset = s.read()?;
        let ySuperscriptYOffset = s.read()?;
        let yStrikeoutSize = s.read()?;
        let yStrikeoutPosition = s.read()?;
        let sFamilyClass = s.read()?;
        let panose = s.read_bytes(10)?.try_into().unwrap();
        let ulUnicodeRange1 = s.read()?;
        let ulUnicodeRange2 = s.read()?;
        let ulUnicodeRange3 = s.read()?;
        let ulUnicodeRange4 = s.read()?;
        let achVendID = s.read()?;
        let fsSelection = s.read()?;
        let usFirstCharIndex = s.read()?;
        let usLastCharIndex = s.read()?;
        let sTypoAscender = s.read();
        let sTypoDescender = s.read();
        let sTypoLineGap = s.read();
        let usWinAscent = s.read();
        let usWinDescent = s.read();
        let ulCodePageRange1 = s.read();
        let ulCodePageRange2 = s.read();
        let sxHeight = s.read();
        let sCapHeight = s.read();
        let usDefaultChar = s.read();
        let usBreakChar = s.read();
        let usMaxContext = s.read();
        let usLowerOpticalPointSize = s.read();
        let usUpperOpticalPointSize = s.read();
        Some(Self {
            version,
            xAvgCharWidth,
            usWeightClass,
            usWidthClass,
            fsType,
            ySubscriptXSize,
            ySubscriptYSize,
            ySubscriptXOffset,
            ySubscriptYOffset,
            ySuperscriptXSize,
            ySuperscriptYSize,
            ySuperscriptXOffset,
            ySuperscriptYOffset,
            yStrikeoutSize,
            yStrikeoutPosition,
            sFamilyClass,
            panose,
            ulUnicodeRange1,
            ulUnicodeRange2,
            ulUnicodeRange3,
            ulUnicodeRange4,
            achVendID,
            fsSelection,
            usFirstCharIndex,
            usLastCharIndex,
            sTypoAscender,
            sTypoDescender,
            sTypoLineGap,
            usWinAscent,
            usWinDescent,
            ulCodePageRange1,
            ulCodePageRange2,
            sxHeight,
            sCapHeight,
            usDefaultChar,
            usBreakChar,
            usMaxContext,
            usLowerOpticalPointSize,
            usUpperOpticalPointSize,
        })
    }
}
