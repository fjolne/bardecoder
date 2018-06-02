use image::DynamicImage;
use image::GrayImage;

use failure::Error;

use decode::{Decode, QRDecoder};
use detect::{Detect, LineScan, Location};
use extract::{Extract, QRExtractor};
use prepare::{BlockedMean, Prepare};

use util::qr::{QRData, QRError, QRLocation};

pub struct Decoder<IMG, PREPD> {
    prepare: Box<Prepare<IMG, PREPD>>,
    detect: Box<Detect<PREPD>>,
    qr: ExtractDecode<PREPD, QRLocation, QRData, QRError>,
}

impl<IMG, PREPD> Decoder<IMG, PREPD> {
    pub fn decode(&self, source: IMG) -> Vec<Result<String, Error>> {
        let prepared = self.prepare.prepare(source);
        let locations = self.detect.detect(&prepared);

        if locations.is_empty() {
            return vec![];
        }

        let mut allDecoded = vec![];

        for location in locations {
            match location {
                Location::QR(qrloc) => {
                    let extracted = self.qr.extract.extract(&prepared, qrloc);
                    let decoded = self.qr.decode.decode(extracted);

                    allDecoded.push(decoded.or_else(|err| Err(Error::from(err))));
                }
            }
        }

        allDecoded
    }
}

/// Create a default Decoder
///
/// It will use the following components:
///
/// * prepare: BlockedMean
/// * detect: LineScan
/// * extract: QRExtractor
/// * decode: QRDecoder
///
/// This is meant to provide a good balance between speed and accuracy
pub fn default_decoder() -> Decoder<DynamicImage, GrayImage> {
    default_builder().build()
}

/// Builder struct to create a Decoder
///
/// Required elements are:
///
/// * Prepare
/// * Detect
/// * Extract
/// * Decode
pub struct DecoderBuilder<IMG, PREPD> {
    prepare: Option<Box<Prepare<IMG, PREPD>>>,
    detect: Option<Box<Detect<PREPD>>>,
    qr: Option<ExtractDecode<PREPD, QRLocation, QRData, QRError>>,
}

impl<IMG, PREPD> DecoderBuilder<IMG, PREPD> {
    /// Constructor; all fields initialized as None
    pub fn new() -> DecoderBuilder<IMG, PREPD> {
        DecoderBuilder {
            prepare: None,
            detect: None,
            qr: None,
        }
    }

    pub fn prepare(
        &mut self,
        prepare: Box<Prepare<IMG, PREPD>>,
    ) -> &mut DecoderBuilder<IMG, PREPD> {
        self.prepare = Some(prepare);
        self
    }

    pub fn detect(&mut self, detect: Box<Detect<PREPD>>) -> &mut DecoderBuilder<IMG, PREPD> {
        self.detect = Some(detect);
        self
    }

    pub fn qr(
        &mut self,
        extract: Box<Extract<PREPD, QRLocation, QRData, QRError>>,
        decode: Box<Decode<QRData, QRError>>,
    ) -> &mut DecoderBuilder<IMG, PREPD> {
        self.qr = Some(ExtractDecode { extract, decode });
        self
    }

    /// Build actual Decoder
    ///
    /// # Panics
    ///
    /// Will panic if any of the required components are missing
    pub fn build(self) -> Decoder<IMG, PREPD> {
        if self.prepare.is_none() {
            panic!("Cannot build Decoder without Prepare component");
        }

        if self.detect.is_none() {
            panic!("Cannot build Decoder without Detect component");
        }

        Decoder {
            prepare: self.prepare.unwrap(),
            detect: self.detect.unwrap(),
            qr: self.qr.unwrap(),
        }
    }
}

/// Create a default DecoderBuilder
///
/// It will use the following components:
///
/// * prepare: BlockedMean
/// * locate: LineScan
/// * extract: QRExtractor
/// * decode: QRDecoder
///
/// The builder can then be customised before creating the Decoder
pub fn default_builder() -> DecoderBuilder<DynamicImage, GrayImage> {
    let mut db = DecoderBuilder::new();

    db.prepare(Box::new(BlockedMean::new(5, 7)));
    db.detect(Box::new(LineScan::new()));
    db.qr(Box::new(QRExtractor::new()), Box::new(QRDecoder::new()));

    db
}

struct ExtractDecode<PREPD, LOC, DATA, ERROR> {
    extract: Box<Extract<PREPD, LOC, DATA, ERROR>>,
    decode: Box<Decode<DATA, ERROR>>,
}
