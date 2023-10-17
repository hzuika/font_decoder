use core::fmt;

#[derive(Clone, Copy)]
pub enum PlatformID {
    Unicode(u16),
    Mac(u16),
    Win(u16),
}

impl PlatformID {
    pub fn new(platform_id: u16) -> Self {
        match platform_id {
            0 => Self::Unicode(platform_id),
            1 => Self::Mac(platform_id),
            3 => Self::Win(platform_id),
            _ => panic!("invalid platform id {}", platform_id),
        }
    }

    pub fn to_id(&self) -> u16 {
        match self {
            Self::Unicode(id) => *id,
            Self::Mac(id) => *id,
            Self::Win(id) => *id,
        }
    }

    pub fn to_name(&self) -> &'static str {
        match self {
            Self::Unicode(_) => "Unicode",
            Self::Mac(_) => "Mac",
            Self::Win(_) => "Win",
        }
    }
}

impl fmt::Debug for PlatformID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.to_id(), self.to_name())
    }
}

impl fmt::Display for PlatformID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_name())
    }
}

#[derive(Clone, Copy)]
pub enum EncodingID {
    Unicode(UnicodeEncodingID),
    Mac(MacEncodingID),
    Win(WinEncodingID),
}

impl EncodingID {
    pub fn new(encoding_id: u16, platform_id: u16) -> Self {
        match platform_id {
            0 => Self::Unicode(UnicodeEncodingID(encoding_id)),
            1 => Self::Mac(MacEncodingID(encoding_id)),
            3 => Self::Win(WinEncodingID(encoding_id)),
            _ => panic!("invalid platform id {}", platform_id),
        }
    }
}

impl fmt::Debug for EncodingID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unicode(id) => <UnicodeEncodingID as fmt::Debug>::fmt(id, f),
            Self::Mac(id) => <MacEncodingID as fmt::Debug>::fmt(id, f),
            Self::Win(id) => <WinEncodingID as fmt::Debug>::fmt(id, f),
        }
    }
}

impl fmt::Display for EncodingID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unicode(id) => <UnicodeEncodingID as fmt::Display>::fmt(id, f),
            Self::Mac(id) => <MacEncodingID as fmt::Display>::fmt(id, f),
            Self::Win(id) => <WinEncodingID as fmt::Display>::fmt(id, f),
        }
    }
}

#[derive(Clone, Copy)]
pub enum LanguageID {
    Unicode,
    Mac(MacLanguageID),
    Win(WinLanguageID),
}

impl LanguageID {
    pub fn new(encoding_id: u16, platform_id: u16) -> Self {
        match platform_id {
            0 => Self::Unicode,
            1 => Self::Mac(MacLanguageID(encoding_id)),
            3 => Self::Win(WinLanguageID(encoding_id)),
            _ => panic!("invalid platform id {}", platform_id),
        }
    }
}

impl fmt::Debug for LanguageID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unicode => write!(f, "language id is none"),
            Self::Mac(id) => <MacLanguageID as fmt::Debug>::fmt(id, f),
            Self::Win(id) => <WinLanguageID as fmt::Debug>::fmt(id, f),
        }
    }
}

impl fmt::Display for LanguageID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unicode => write!(f, "language id is none"),
            Self::Mac(id) => <MacLanguageID as fmt::Display>::fmt(id, f),
            Self::Win(id) => <WinLanguageID as fmt::Display>::fmt(id, f),
        }
    }
}

#[derive(Clone, Copy)]
pub struct UnicodeEncodingID(pub u16);
impl UnicodeEncodingID {
    pub fn to_name(&self) -> &'static str {
        match self.0 {
            0 => "Unicode 1.0",
            1 => "Unicode 1.1",
            2 => "ISO/IEC 10646",
            3 => "Unicode 2.0 BMP",
            4 => "Unicode 2.0 full",
            _ => panic!("invalid encoding id {}", self.0),
        }
    }
}

impl fmt::Debug for UnicodeEncodingID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.0, self.to_name())
    }
}

impl fmt::Display for UnicodeEncodingID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_name())
    }
}

// TextEncodingBase と関係する値．
// https://developer.apple.com/documentation/coreservices/textencodingbase
#[derive(Clone, Copy)]
pub struct MacEncodingID(pub u16);
impl MacEncodingID {
    pub fn to_name(&self) -> &'static str {
        match self.0 {
            0 => "Roman",                 // kTextEncodingMacRoman
            1 => "Japanese",              // kTextEncodingMacJapanese
            2 => "Chinese (Traditional)", // kTextEncodingMacChineseTrad
            3 => "Korean",                // kTextEncodingMacKorean
            4 => "Arabic",                // kTextEncodingMacArabic
            5 => "Hebrew",                // kTextEncodingMacHebrew
            6 => "Greek",                 // kTextEncodingMacGreek
            7 => "Russian",               // kTextEncodingMacCyrillic
            8 => "RSymbol",
            9 => "Devanagari",            // kTextEncodingMacDevanagari
            10 => "Gurmukhi",             // kTextEncodingMacGurmukhi
            11 => "Gujarati",             // kTextEncodingMacGujarati
            12 => "Oriya",                // kTextEncodingMacOriya
            13 => "Bengali",              // kTextEncodingMacBengali
            14 => "Tamil",                // kTextEncodingMacTamil
            15 => "Telugu",               // kTextEncodingMacTelugu
            16 => "Kannada",              // kTextEncodingMacKannada
            17 => "Malayalam",            // kTextEncodingMacMalayalam
            18 => "Sinhalese",            // kTextEncodingMacSinhalese
            19 => "Burmese",              // kTextEncodingMacBurmese
            20 => "Khmer",                // kTextEncodingMacKhmer
            21 => "Thai",                 // kTextEncodingMacThai
            22 => "Laotian",              // kTextEncodingMacLaotian
            23 => "Georgian",             // kTextEncodingMacGeorgian
            24 => "Armenian",             // kTextEncodingMacArmenian
            25 => "Chinese (Simplified)", // kTextEncodingMacChineseSimp
            26 => "Tibetan",              // kTextEncodingMacTibetan
            27 => "Mongolian",            // kTextEncodingMacMongolian
            28 => "Geez",                 // kTextEncodingMacEthiopic
            29 => "Slavic",               // kTextEncodingMacCentralEurRoman
            30 => "Vietnamese",           // kTextEncodingMacVietnamese
            31 => "Sindhi",               // kTextEncodingMacExtArabic
            32 => "Uninterpreted",
            _ => panic!("invalid encoding id {}", self.0),
        }
    }
}

impl fmt::Debug for MacEncodingID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.0, self.to_name())
    }
}

impl fmt::Display for MacEncodingID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_name())
    }
}

#[derive(Clone, Copy)]
pub struct MacLanguageID(pub u16);

impl MacLanguageID {
    pub fn to_name(&self) -> &'static str {
        match self.0 {
            0 => "English",
            1 => "French",
            2 => "German",
            3 => "Italian",
            4 => "Dutch",
            5 => "Swedish",
            6 => "Spanish",
            7 => "Danish",
            8 => "Portuguese",
            9 => "Norwegian",
            10 => "Hebrew",
            11 => "Japanese",
            12 => "Arabic",
            13 => "Finnish",
            14 => "Greek",
            15 => "Icelandic",
            16 => "Maltese",
            17 => "Turkish",
            18 => "Croatian",
            19 => "Chinese (Traditional)",
            20 => "Urdu",
            21 => "Hindi",
            22 => "Thai",
            23 => "Korean",
            24 => "Lithuanian",
            25 => "Polish",
            26 => "Hungarian",
            27 => "Estonian",
            28 => "Latvian",
            29 => "Sami",
            30 => "Faroese",
            31 => "Farsi/Persian",
            32 => "Russian",
            33 => "Chinese (Simplified)",
            34 => "Flemish",
            35 => "Irish Gaelic",
            36 => "Albanian",
            37 => "Romanian",
            38 => "Czech",
            39 => "Slovak",
            40 => "Slovenian",
            41 => "Yiddish",
            42 => "Serbian",
            43 => "Macedonian",
            44 => "Bulgarian",
            45 => "Ukrainian",
            46 => "Byelorussian",
            47 => "Uzbek",
            48 => "Kazakh",
            49 => "Azerbaijani (Cyrillic script)",
            50 => "Azerbaijani (Arabic script)",
            51 => "Armenian",
            52 => "Georgian",
            53 => "Moldavian",
            54 => "Kirghiz",
            55 => "Tajiki",
            56 => "Turkmen",
            57 => "Mongolian (Mongolian script)",
            58 => "Mongolian (Cyrillic script)",
            59 => "Pashto",
            60 => "Kurdish",
            61 => "Kashmiri",
            62 => "Sindhi",
            63 => "Tibetan",
            64 => "Nepali",
            65 => "Sanskrit",
            66 => "Marathi",
            67 => "Bengali",
            68 => "Assamese",
            69 => "Gujarati",
            70 => "Punjabi",
            71 => "Oriya",
            72 => "Malayalam",
            73 => "Kannada",
            74 => "Tamil",
            75 => "Telugu",
            76 => "Sinhalese",
            77 => "Burmese",
            78 => "Khmer",
            79 => "Lao",
            80 => "Vietnamese",
            81 => "Indonesian",
            82 => "Tagalog",
            83 => "Malay (Roman script)",
            84 => "Malay (Arabic script)",
            85 => "Amharic",
            86 => "Tigrinya",
            87 => "Galla",
            88 => "Somali",
            89 => "Swahili",
            90 => "Kinyarwanda/Ruanda",
            91 => "Rundi",
            92 => "Nyanja/Chewa",
            93 => "Malagasy",
            94 => "Esperanto",
            128 => "Welsh",
            129 => "Basque",
            130 => "Catalan",
            131 => "Latin",
            132 => "Quechua",
            133 => "Guarani",
            134 => "Aymara",
            135 => "Tatar",
            136 => "Uighur",
            137 => "Dzongkha",
            138 => "Javanese (Roman script)",
            139 => "Sundanese (Roman script)",
            140 => "Galician",
            141 => "Afrikaans",
            142 => "Breton",
            143 => "Inuktitut",
            144 => "Scottish Gaelic",
            145 => "Manx Gaelic",
            146 => "Irish Gaelic (with dot above)",
            147 => "Tongan",
            148 => "Greek (polytonic)",
            149 => "Greenlandic",
            150 => "Azerbaijani (Roman script)",
            _ => panic!("invalid language id {}", self.0),
        }
    }
}

impl fmt::Debug for MacLanguageID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.0, self.to_name())
    }
}

impl fmt::Display for MacLanguageID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_name())
    }
}

#[derive(Clone, Copy)]
pub struct WinEncodingID(pub u16);
impl WinEncodingID {
    pub fn to_name(&self) -> &'static str {
        match self.0 {
            0 => "Symbol",
            1 => "Unicode BMP",
            2 => "ShiftJIS",
            3 => "PRC",
            4 => "Big5",
            5 => "Wansung",
            6 => "Johab",
            7 => "Reserved",
            8 => "Reserved",
            9 => "Reserved",
            10 => "Unicode Full",
            _ => panic!("invalid encoding id {}", self.0),
        }
    }
}

impl fmt::Debug for WinEncodingID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.0, self.to_name())
    }
}

impl fmt::Display for WinEncodingID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_name())
    }
}

// https://learn.microsoft.com/ja-jp/windows/win32/intl/language-identifiers
// +-------------------------+-------------------------+
// |     SubLanguage ID      |   Primary Language ID   |
// +-------------------------+-------------------------+
// 15                    10  9                         0   bit
// https://learn.microsoft.com/ja-jp/openspecs/windows_protocols/ms-lcid/70feba9f-294e-491e-b6eb-56532684c37f
#[derive(Clone, Copy)]
pub struct WinLanguageID(pub u16);
impl WinLanguageID {
    pub fn to_name(&self) -> &'static str {
        match self.0 {
            0x0436 => "Afrikaans South Africa",
            0x041C => "Albanian Albania",
            0x0484 => "Alsatian France",
            0x045E => "Amharic Ethiopia",
            0x1401 => "Arabic Algeria",
            0x3C01 => "Arabic Bahrain",
            0x0C01 => "Arabic Egypt",
            0x0801 => "Arabic Iraq",
            0x2C01 => "Arabic Jordan",
            0x3401 => "Arabic Kuwait",
            0x3001 => "Arabic Lebanon",
            0x1001 => "Arabic Libya",
            0x1801 => "Arabic Morocco",
            0x2001 => "Arabic Oman",
            0x4001 => "Arabic Qatar",
            0x0401 => "Arabic Saudi Arabia",
            0x2801 => "Arabic Syria",
            0x1C01 => "Arabic Tunisia",
            0x3801 => "Arabic U.A.E.",
            0x2401 => "Arabic Yemen",
            0x042B => "Armenian Armenia",
            0x044D => "Assamese India",
            0x082C => "Azeri (Cyrillic) Azerbaijan",
            0x042C => "Azeri (Latin) Azerbaijan",
            0x046D => "Bashkir Russia",
            0x042D => "Basque Basque",
            0x0423 => "Belarusian Belarus",
            0x0845 => "Bengali Bangladesh",
            0x0445 => "Bengali India",
            0x201A => "Bosnian (Cyrillic) Bosnia and Herzegovina",
            0x141A => "Bosnian (Latin) Bosnia and Herzegovina",
            0x047E => "Breton France",
            0x0402 => "Bulgarian Bulgaria",
            0x0403 => "Catalan Catalan",
            0x0C04 => "Chinese Hong Kong S.A.R.",
            0x1404 => "Chinese Macao S.A.R.",
            0x0804 => "Chinese People’s Republic of China",
            0x1004 => "Chinese Singapore",
            0x0404 => "Chinese Taiwan",
            0x0483 => "Corsican France",
            0x041A => "Croatian Croatia",
            0x101A => "Croatian (Latin) Bosnia and Herzegovina",
            0x0405 => "Czech Czech Republic",
            0x0406 => "Danish Denmark",
            0x048C => "Dari Afghanistan",
            0x0465 => "Divehi Maldives",
            0x0813 => "Dutch Belgium",
            0x0413 => "Dutch Netherlands",
            0x0C09 => "English Australia",
            0x2809 => "English Belize",
            0x1009 => "English Canada",
            0x2409 => "English Caribbean",
            0x4009 => "English India",
            0x1809 => "English Ireland",
            0x2009 => "English Jamaica",
            0x4409 => "English Malaysia",
            0x1409 => "English New Zealand",
            0x3409 => "English Republic of the Philippines",
            0x4809 => "English Singapore",
            0x1C09 => "English South Africa",
            0x2C09 => "English Trinidad and Tobago",
            0x0809 => "English United Kingdom",
            0x0409 => "English United States",
            0x3009 => "English Zimbabwe",
            0x0425 => "Estonian Estonia",
            0x0438 => "Faroese Faroe Islands",
            0x0464 => "Filipino Philippines",
            0x040B => "Finnish Finland",
            0x080C => "French Belgium",
            0x0C0C => "French Canada",
            0x040C => "French France",
            0x140c => "French Luxembourg",
            0x180C => "French Principality of Monaco",
            0x100C => "French Switzerland",
            0x0462 => "Frisian Netherlands",
            0x0456 => "Galician Galician",
            0x0437 => "Georgian Georgia",
            0x0C07 => "German Austria",
            0x0407 => "German Germany",
            0x1407 => "German Liechtenstein",
            0x1007 => "German Luxembourg",
            0x0807 => "German Switzerland",
            0x0408 => "Greek Greece",
            0x046F => "Greenlandic Greenland",
            0x0447 => "Gujarati India",
            0x0468 => "Hausa (Latin) Nigeria",
            0x040D => "Hebrew Israel",
            0x0439 => "Hindi India",
            0x040E => "Hungarian Hungary",
            0x040F => "Icelandic Iceland",
            0x0470 => "Igbo Nigeria",
            0x0421 => "Indonesian Indonesia",
            0x045D => "Inuktitut Canada",
            0x085D => "Inuktitut (Latin) Canada",
            0x083C => "Irish Ireland",
            0x0434 => "isiXhosa South Africa",
            0x0435 => "isiZulu South Africa",
            0x0410 => "Italian Italy",
            0x0810 => "Italian Switzerland",
            0x0411 => "Japanese Japan",
            0x044B => "Kannada India",
            0x043F => "Kazakh Kazakhstan",
            0x0453 => "Khmer Cambodia",
            0x0486 => "K’iche Guatemala",
            0x0487 => "Kinyarwanda Rwanda",
            0x0441 => "Kiswahili Kenya",
            0x0457 => "Konkani India",
            0x0412 => "Korean Korea",
            0x0440 => "Kyrgyz Kyrgyzstan",
            0x0454 => "Lao Lao P.D.R.",
            0x0426 => "Latvian Latvia",
            0x0427 => "Lithuanian Lithuania",
            0x082E => "Lower Sorbian Germany",
            0x046E => "Luxembourgish Luxembourg",
            0x042F => "Macedonian North Macedonia",
            0x083E => "Malay Brunei Darussalam",
            0x043E => "Malay Malaysia",
            0x044C => "Malayalam India",
            0x043A => "Maltese Malta",
            0x0481 => "Maori New Zealand",
            0x047A => "Mapudungun Chile",
            0x044E => "Marathi India",
            0x047C => "Mohawk Mohawk",
            0x0450 => "Mongolian (Cyrillic) Mongolia",
            0x0850 => "Mongolian (Traditional) People’s Republic of China",
            0x0461 => "Nepali Nepal",
            0x0414 => "Norwegian (Bokmal) Norway",
            0x0814 => "Norwegian (Nynorsk) Norway",
            0x0482 => "Occitan France",
            0x0448 => "Odia (formerly Oriya) India",
            0x0463 => "Pashto Afghanistan",
            0x0415 => "Polish Poland",
            0x0416 => "Portuguese Brazil",
            0x0816 => "Portuguese Portugal",
            0x0446 => "Punjabi India",
            0x046B => "Quechua Bolivia",
            0x086B => "Quechua Ecuador",
            0x0C6B => "Quechua Peru",
            0x0418 => "Romanian Romania",
            0x0417 => "Romansh Switzerland",
            0x0419 => "Russian Russia",
            0x243B => "Sami (Inari) Finland",
            0x103B => "Sami (Lule) Norway",
            0x143B => "Sami (Lule) Sweden",
            0x0C3B => "Sami (Northern) Finland",
            0x043B => "Sami (Northern) Norway",
            0x083B => "Sami (Northern) Sweden",
            0x203B => "Sami (Skolt) Finland",
            0x183B => "Sami (Southern) Norway",
            0x1C3B => "Sami (Southern) Sweden",
            0x044F => "Sanskrit India",
            0x1C1A => "Serbian (Cyrillic) Bosnia and Herzegovina",
            0x0C1A => "Serbian (Cyrillic) Serbia",
            0x181A => "Serbian (Latin) Bosnia and Herzegovina",
            0x081A => "Serbian (Latin) Serbia",
            0x046C => "Sesotho sa Leboa South Africa",
            0x0432 => "Setswana South Africa",
            0x045B => "Sinhala Sri Lanka",
            0x041B => "Slovak Slovakia",
            0x0424 => "Slovenian Slovenia",
            0x2C0A => "Spanish Argentina",
            0x400A => "Spanish Bolivia",
            0x340A => "Spanish Chile",
            0x240A => "Spanish Colombia",
            0x140A => "Spanish Costa Rica",
            0x1C0A => "Spanish Dominican Republic",
            0x300A => "Spanish Ecuador",
            0x440A => "Spanish El Salvador",
            0x100A => "Spanish Guatemala",
            0x480A => "Spanish Honduras",
            0x080A => "Spanish Mexico",
            0x4C0A => "Spanish Nicaragua",
            0x180A => "Spanish Panama",
            0x3C0A => "Spanish Paraguay",
            0x280A => "Spanish Peru",
            0x500A => "Spanish Puerto Rico",
            0x0C0A => "Spanish (Modern Sort) Spain",
            0x040A => "Spanish (Traditional Sort) Spain",
            0x540A => "Spanish United States",
            0x380A => "Spanish Uruguay",
            0x200A => "Spanish Venezuela",
            0x081D => "Swedish Finland",
            0x041D => "Swedish Sweden",
            0x045A => "Syriac Syria",
            0x0428 => "Tajik (Cyrillic) Tajikistan",
            0x085F => "Tamazight (Latin) Algeria",
            0x0449 => "Tamil India",
            0x0444 => "Tatar Russia",
            0x044A => "Telugu India",
            0x041E => "Thai Thailand",
            0x0451 => "Tibetan PRC",
            0x041F => "Turkish Turkey",
            0x0442 => "Turkmen Turkmenistan",
            0x0480 => "Uighur PRC",
            0x0422 => "Ukrainian Ukraine",
            0x042E => "Upper Sorbian Germany",
            0x0420 => "Urdu Islamic Republic of Pakistan",
            0x0843 => "Uzbek (Cyrillic) Uzbekistan",
            0x0443 => "Uzbek (Latin) Uzbekistan",
            0x042A => "Vietnamese Vietnam",
            0x0452 => "Welsh United Kingdom",
            0x0488 => "Wolof Senegal",
            0x0485 => "Yakut Russia",
            0x0478 => "Yi PRC",
            0x046A => "Yoruba Nigeria",
            _ => panic!("invalid language id {}", self.0),
        }
    }

    // LCIDToLocaleName() の変換に対応している．
    pub fn to_tag(id: u16) -> &'static str {
        match id {
            0x0436 => "af-ZA",
            0x041C => "sq-AL",
            0x0484 => "gsw-FR",
            0x045E => "am-ET",
            0x1401 => "ar-DZ",
            0x3C01 => "ar-BH",
            0x0C01 => "ar-EG",
            0x0801 => "ar-IQ",
            0x2C01 => "ar-JO",
            0x3401 => "ar-KW",
            0x3001 => "ar-LB",
            0x1001 => "ar-LY",
            0x1801 => "ar-MA",
            0x2001 => "ar-OM",
            0x4001 => "ar-QA",
            0x0401 => "ar-SA",
            0x2801 => "ar-SY",
            0x1C01 => "ar-TN",
            0x3801 => "ar-AE",
            0x2401 => "ar-YE",
            0x042B => "hy-AM",
            0x044D => "as-IN",
            0x082C => "az-Cyrl-AZ",
            0x042C => "az-Latn-AZ",
            0x046D => "ba-RU",
            0x042D => "eu-ES",
            0x0423 => "be-BY",
            0x0845 => "bn-BD",
            0x0445 => "bn-IN",
            0x201A => "bs-Cyrl-BA",
            0x141A => "bs-Latn-BA",
            0x047E => "br-FR",
            0x0402 => "bg-BG",
            0x0403 => "ca-ES",
            0x0C04 => "zh-HK",
            0x1404 => "zh-MO",
            0x0804 => "zh-CN",
            0x1004 => "zh-SG",
            0x0404 => "zh-TW",
            0x0483 => "co-FR",
            0x041A => "hr-HR",
            0x101A => "hr-BA",
            0x0405 => "cs-CZ",
            0x0406 => "da-DK",
            0x048C => "fa-AF",
            0x0465 => "dv-MV",
            0x0813 => "nl-BE",
            0x0413 => "nl-NL",
            0x0C09 => "en-AU",
            0x2809 => "en-BZ",
            0x1009 => "en-CA",
            0x2409 => "en-029",
            0x4009 => "en-IN",
            0x1809 => "en-IE",
            0x2009 => "en-JM",
            0x4409 => "en-MY",
            0x1409 => "en-NZ",
            0x3409 => "en-PH",
            0x4809 => "en-SG",
            0x1C09 => "en-ZA",
            0x2C09 => "en-TT",
            0x0809 => "en-GB",
            0x0409 => "en-US",
            0x3009 => "en-ZW",
            0x0425 => "et-EE",
            0x0438 => "fo-FO",
            0x0464 => "fil-PH",
            0x040B => "fi-FI",
            0x080C => "fr-BE",
            0x0C0C => "fr-CA",
            0x040C => "fr-FR",
            0x140C => "fr-LU",
            0x180C => "fr-MC",
            0x100C => "fr-CH",
            0x0462 => "fy-NL",
            0x0456 => "gl-ES",
            0x0437 => "ka-GE",
            0x0C07 => "de-AT",
            0x0407 => "de-DE",
            0x1407 => "de-LI",
            0x1007 => "de-LU",
            0x0807 => "de-CH",
            0x0408 => "el-GR",
            0x046F => "kl-GL",
            0x0447 => "gu-IN",
            0x0468 => "ha-Latn-NG",
            0x040D => "he-IL",
            0x0439 => "hi-IN",
            0x040E => "hu-HU",
            0x040F => "is-IS",
            0x0470 => "ig-NG",
            0x0421 => "id-ID",
            0x045D => "iu-Cans-CA",
            0x085D => "iu-Latn-CA",
            0x083C => "ga-IE",
            0x0434 => "xh-ZA",
            0x0435 => "zu-ZA",
            0x0410 => "it-IT",
            0x0810 => "it-CH",
            0x0411 => "ja-JP",
            0x044B => "kn-IN",
            0x043F => "kk-KZ",
            0x0453 => "km-KH",
            0x0486 => "quc-Latn-GT",
            0x0487 => "rw-RW",
            0x0441 => "sw-KE",
            0x0457 => "kok-IN",
            0x0412 => "ko-KR",
            0x0440 => "ky-KG",
            0x0454 => "lo-LA",
            0x0426 => "lv-LV",
            0x0427 => "lt-LT",
            0x082E => "dsb-DE",
            0x046E => "lb-LU",
            0x042F => "mk-MK",
            0x083E => "ms-BN",
            0x043E => "ms-MY",
            0x044C => "ml-IN",
            0x043A => "mt-MT",
            0x0481 => "mi-NZ",
            0x047A => "arn-CL",
            0x044E => "mr-IN",
            0x047C => "moh-CA",
            0x0450 => "mn-MN",
            0x0850 => "mn-Mong-CN",
            0x0461 => "ne-NP",
            0x0414 => "nb-NO",
            0x0814 => "nn-NO",
            0x0482 => "oc-FR",
            0x0448 => "or-IN",
            0x0463 => "ps-AF",
            0x0415 => "pl-PL",
            0x0416 => "pt-BR",
            0x0816 => "pt-PT",
            0x0446 => "pa-IN",
            0x046B => "quz-BO",
            0x086B => "quz-EC",
            0x0C6B => "quz-PE",
            0x0418 => "ro-RO",
            0x0417 => "rm-CH",
            0x0419 => "ru-RU",
            0x243B => "smn-FI",
            0x103B => "smj-NO",
            0x143B => "smj-SE",
            0x0C3B => "se-FI",
            0x043B => "se-NO",
            0x083B => "se-SE",
            0x203B => "sms-FI",
            0x183B => "sma-NO",
            0x1C3B => "sma-SE",
            0x044F => "sa-IN",
            0x1C1A => "sr-Cyrl-BA",
            0x0C1A => "sr-Cyrl-CS",
            0x181A => "sr-Latn-BA",
            0x081A => "sr-Latn-CS",
            0x046C => "nso-ZA",
            0x0432 => "tn-ZA",
            0x045B => "si-LK",
            0x041B => "sk-SK",
            0x0424 => "sl-SI",
            0x2C0A => "es-AR",
            0x400A => "es-BO",
            0x340A => "es-CL",
            0x240A => "es-CO",
            0x140A => "es-CR",
            0x1C0A => "es-DO",
            0x300A => "es-EC",
            0x440A => "es-SV",
            0x100A => "es-GT",
            0x480A => "es-HN",
            0x080A => "es-MX",
            0x4C0A => "es-NI",
            0x180A => "es-PA",
            0x3C0A => "es-PY",
            0x280A => "es-PE",
            0x500A => "es-PR",
            0x0C0A => "es-ES",
            0x040A => "es-ES_tradnl",
            0x540A => "es-US",
            0x380A => "es-UY",
            0x200A => "es-VE",
            0x081D => "sv-FI",
            0x041D => "sv-SE",
            0x045A => "syr-SY",
            0x0428 => "tg-Cyrl-TJ",
            0x085F => "tzm-Latn-DZ",
            0x0449 => "ta-IN",
            0x0444 => "tt-RU",
            0x044A => "te-IN",
            0x041E => "th-TH",
            0x0451 => "bo-CN",
            0x041F => "tr-TR",
            0x0442 => "tk-TM",
            0x0480 => "ug-CN",
            0x0422 => "uk-UA",
            0x042E => "hsb-DE",
            0x0420 => "ur-PK",
            0x0843 => "uz-Cyrl-UZ",
            0x0443 => "uz-Latn-UZ",
            0x042A => "vi-VN",
            0x0452 => "cy-GB",
            0x0488 => "wo-SN",
            0x0485 => "sah-RU",
            0x0478 => "ii-CN",
            0x046A => "yo-NG",
            _ => panic!("invalid language id {}", id),
        }
    }
}

impl fmt::Debug for WinLanguageID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0} (= 0x{0:x}, {1})", self.0, self.to_name())
    }
}

impl fmt::Display for WinLanguageID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_name())
    }
}

#[derive(PartialEq, Clone, Copy, Hash)]
pub struct NameID(pub u16);
impl NameID {
    pub fn to_name(&self) -> &'static str {
        match self.0 {
            0 => "Copyright",
            1 => "Family name",
            2 => "Subfamily name",
            3 => "Unique font identifier",
            4 => "Full name",
            5 => "Version",
            6 => "PostScript name",
            7 => "Trademark",
            8 => "Manufacturer name",
            9 => "Designer",
            10 => "Description",
            11 => "URL Vendor",
            12 => "URL Designer",
            13 => "License",
            14 => "License Info URL",
            15 => "Reserved",
            16 => "Typographic Family name",
            17 => "Typographic Subfamily name",
            18 => "Compatible Full",
            19 => "Sample text",
            20 => "PostScript CID findfont name",
            21 => "WWS Family name",
            22 => "WWS Subfamily name",
            23 => "Light Background Palette",
            24 => "Dark Background Palette",
            25 => "Variations PostScript Name Prefix",
            26..=255 => "Reserved",
            _ => "Unique",
        }
    }
}

impl fmt::Debug for NameID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.0, self.to_name())
    }
}

impl fmt::Display for NameID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_name())
    }
}
