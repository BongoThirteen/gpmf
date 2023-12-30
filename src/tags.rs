/// The FourCC key of the data
///
/// There are some undocumented tags present in GPMF data.
/// Currently warnings are logged for unsupported tags.
#[derive(Debug, Clone, PartialEq, EnumString, EnumIter, Display)]
pub enum Tag {
    ///unique device source for metadata
    /// Each connected device starts with DEVC. A GoPro camera or Karma drone would have their own DEVC for nested metadata to follow. |
    #[strum(serialize = "DEVC", to_string = "Device")]
    DEVC,

    /// device/track ID
    /// Auto generated unique-ID for managing a large number of connect devices, camera, karma and external BLE devices |
    #[strum(serialize = "DVID", to_string = "Device ID")]
    DVID,

    /// device name
    /// Display name of the device like &quot;Karma 1.0&quot;, this is for communicating to the user the data recorded, so it should be informative. |
    #[strum(serialize = "DVNM", to_string = "Device Name")]
    DVNM,

    ///Nested signal stream of metadata/telemetry
    ///Metadata streams are each nested with STRM
    #[strum(serialize = "STRM", to_string = "Stream")]
    STRM,

    ///Stream name
    /// Display name for a stream like &quot;GPS RAW&quot;, this is for communicating to the user the data recorded, so it should be informative. |
    #[strum(serialize = "STNM", to_string = "Stream Name")]
    STNM,

    ///Comments for any stream
    /// Add more human readable information about the stream |
    #[strum(serialize = "RMRK", to_string = "Stream Comment")]
    RMRK,

    ///Scaling factor (divisor) | Sensor data often needs to be scaled to be presented with the correct units. SCAL is a divisor. |
    #[strum(serialize = "SCAL", to_string = "Scale Factor")]
    SCAL,

    ///Standard Units (like SI) | If the data can be formatted in GPMF&#39;s standard units, this is best. E.g. acceleration as &quot;m/s²&quot;.  SIUN allows for simple format conversions. |
    #[strum(serialize = "SIUN", to_string = "SI Unit")]
    SIUN,

    /// Display units
    /// While SIUN is preferred, not everything communicates well via standard units. E.g. engine speed as &quot;RPM&quot; is more user friendly than &quot;rad/s&quot;. |
    #[strum(serialize = "UNIT", to_string = "Non SI Unit")]
    UNIT,
    ///Typedefs for complex structures
    /// Not everything has a simple repeating type. For complex structure TYPE is used to describe the data packed within each sample. |
    #[strum(serialize = "TYPE", to_string = "Type")]
    TYPE,
    ///Total Samples delivered | Internal field that counts all the sample delivered since record start, and is automatically computed. |
    #[strum(serialize = "TSMP", to_string = "Total Samples")]
    TSMP,
    ///Time Offset | Rare. An internal field that indicates the data is delayed by 'x' seconds. |
    #[strum(serialize = "TIMO", to_string = "Time Offset")]
    TIMO,
    ///Empty payload count
    /// Internal field that reports the number of payloads that contain no new data. TSMP and EMPT simplify the extraction of clock. |
    #[strum(serialize = "EMPT", to_string = "Empty Payload Count")]
    EMPT,

    /// Start Time
    #[strum(serialize = "TICK", to_string = "Start Timestamp")]
    TICK,
    /// End time
    #[strum(serialize = "TOCK", to_string = "End Timestamp")]
    TOCK,

    /// thermal clock drift for temperature sensitive calibrations.
    #[strum(serialize = "TMPC", to_string = "Temp")]
    TMPC,

    //HERO5 Black and Session
    ///3-axis accelerometer
    // Hero5: Data order Z,X,Y
    // Fusion: Data order -Y,X,Z
    // Hero 6 Data order Y,-X,Z
    #[strum(serialize = "ACCL", to_string = "Accel")]
    ACCL,
    /// 3-axis gyroscope
    // Hero5: Data order Z,X,Y
    // Fusion: Data order -Y,X,Z
    // Hero6 Data order Y,-X,Z
    #[strum(serialize = "GYRO", to_string = "Gyro")]
    GYRO,
    ///Image sensor gain
    #[strum(serialize = "ISOG", to_string = "Image Sensor Gain")]
    ISOG,
    /// Exposure time
    #[strum(serialize = "SHUT", to_string = "Exposure Time")]
    SHUT,

    //HERO5 Black with GPS Enabled Adds
    /// latitude, longitude, altitude (WGS 84), 2D ground speed, and 3D speed
    #[strum(serialize = "GPS5", to_string = "GPS 5")]
    GPS5,

    ///UTC time and data from GPS
    #[strum(serialize = "GPSU", to_string = "GPS UTC")]
    GPSU,
    ///GPS Fix Within the GPS stream: 0 - no lock, 2 or 3 - 2D or 3D Lock
    #[strum(serialize = "GPSF", to_string = "GPS Fix")]
    GPSF,
    ///GPS Precision - Dilution of Precision (DOP x100) Within the GPS stream, under 500 is good
    #[strum(serialize = "GPSP", to_string = "GPS DOP")]
    GPSP,

    //Fusion Adds and Changes
    ///Magnetometer
    //GoPro MAX  Camera pointing direction x,y,z (valid in v2.0 firmware.)
    #[strum(serialize = "MAGN", to_string = "Magnetometer")]
    MAGN,

    //FUSION
    /// Microsecond Timestamps
    #[strum(serialize = "STMP", to_string = "Timestamp")]
    STMP,

    //HERO6 Black
    /// Face detection boundaring boxes
    // Herd6 struct ID,x,y,w,h -- not supported in HEVC modes
    // Hero 7 struct ID,x,y,w,h,unused[17],smile
    // Hero 8 struct ID,x,y,w,h,confidence %,smile %
    // Hero 10 struct ver,confidence %,ID,x,y,w,h,smile %, blink %
    #[strum(serialize = "FACE", to_string = "Face Bbox")]
    FACE,
    /// Faces counted per frame
    #[strum(serialize = "FCNM", to_string = "Face count per Frame")]
    FCNM,

    //#[strum(serialize = "FSTM", to_string = "UNDOCUMENTED Face something ???")]
    //FSTM,
    /// Sensor ISO replaces ISOG, has the same function
    #[strum(serialize = "ISOE", to_string = "Image Sensor Gain E")]
    ISOE,

    /// Auto Low Light frame Duration
    #[strum(serialize = "ALLD", to_string = "Auto Low Light frame Duration")]
    ALLD,

    /// White Balance in Kelvin
    #[strum(serialize = "WBAL", to_string = "White Balance in Kelvin")]
    WBAL,
    /// White Balance RGB gains
    #[strum(serialize = "WRGB", to_string = "White Balance RGB gains")]
    WRGB,

    //HERO7 Black (v1.8)
    /// Luma (Y) Average over the frame
    #[strum(serialize = "YAVG", to_string = "Luma (Y) Average over the frame")]
    YAVG,

    /// Predominant hues over the frame
    // struct ubyte hue, ubyte weight, HSV_Hue = hue x 360/255
    #[strum(serialize = "HUES", to_string = "Predominant hues over the frame")]
    HUES,

    /// Image uniformity
    #[strum(serialize = "UNIF", to_string = "Image uniformity")]
    UNIF,

    /// Scene classifier in probabilities
    /// FourCC scenes: SNOW, URBAn, INDOor, WATR, VEGEtation, BEACh
    #[strum(serialize = "SCEN", to_string = "Scene classifier")]
    SCEN,

    /// Sensor Read Out Time
    #[strum(serialize = "SROT", to_string = "Sensor Read Out Time")]
    SROT,

    // HERO8 Black (v2.5)
    /// Camera ORIentation
    /// Quaternions for the camera orientation since capture start
    #[strum(serialize = "CORI", to_string = "Camera Orientation")]
    CORI,

    // #[strum(
    //     serialize = "ORIO",
    //     to_string = "UNDOCUMENTED: ORIO Camera Orientation ???"
    // )]
    // ORIO,
    // #[strum(
    //     serialize = "ORIN",
    //     to_string = "UNDOCUMENTED: ORIN Camera Orientation ???"
    // )]
    // ORIN,
    /// Image ORIentation
    /// Quaternions for the image orientation relative to the camera body
    #[strum(serialize = "IORI", to_string = "Image Orientation")]
    IORI,

    ///GRAvity Vector
    ///Vector for the direction for gravitiy
    #[strum(serialize = "GRAV", to_string = "Gravity Vector")]
    GRAV,

    ///Wind Processing
    ///marks whether wind processing is active
    #[strum(serialize = "WNDM", to_string = "Wind Processing")]
    WNDM,

    ///Microphone is WET
    ///marks whether some of the microphones are wet
    #[strum(serialize = "MWET", to_string = "Microphone is Wet")]
    MWET,

    /// Audio Levels
    /// RMS and peak audio levels in dBFS
    #[strum(serialize = "AALP", to_string = "Audio Levels (dBFS)")]
    AALP,

    //GoPro MAX (v2.0)
    /// 1-D depth map for the objects seen by the two lenses
    #[strum(serialize = "DISP", to_string = "Depth Map")]
    DISP,

    //HERO9
    /// Main video frame SKiP
    #[strum(serialize = "MSKP", to_string = "Main video frame skip")]
    MSKP,
    /// Low res video frame SKiP
    #[strum(serialize = "LSKP", to_string = "Low res video frame skip")]
    LSKP,

    //HERO11
    /// GPS lat, long, alt, 2D speed, 3D speed, days since 2000, secs since midnight (ms precision), DOP, fix (0, 2D or 3D)
    /// improved precision over GPS5 for time and fix information
    //GPS5 deprecated
    #[strum(serialize = "GPS9", to_string = "GPS 9")]
    GPS9,

    ///  Its data consists of one or more 32-bit integers. The first integer contains the number of available HiLight tags. All subsequent integers resemble an ordered list of HiLight tags. Each HiLight tag is represented as a millisecond value.
    /// <https://superuser.com/questions/881661/how-where-does-a-gopro-camera-store-hilight-tags>
    #[strum(serialize = "HMMT", to_string = "HMMT UNDOCUMENTED HiLights ???")]
    HMMT,

    // #[strum(serialize = "HLMT", to_string = "HLMT UNDOCUMENTED HiLights ???")]
    // HLMT,
    //
    // #[strum(serialize = "MANL", to_string = "MANL UNDOCUMENTED Manual Label ???")]
    // MANL,

    // #[strum(serialize = "MTRX", to_string = "MTRX UNDOCUMENTED Tracks ???")]
    // MTRX,

    // #[strum(serialize = "AGST", to_string = "AGST UNDOCUMENTED ???")]
    // AGST,
    /// Battery Status
    #[strum(serialize = "KBAT", to_string = "KBAT UNDOCUMENTED Battery Status ???")]
    KBAT,

    /// Other custom metadata
    #[strum(default)]
    Other(String),
}
