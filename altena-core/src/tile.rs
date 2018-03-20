//! color, tile

use ansi_term::Style;
use ansi_term::Colour as TermRGB;
use image::{Primitive, Rgba};
use num_traits::ToPrimitive;
use rect_iter::{Get2D, GetMut2D};
use std::convert;
use std::fmt;
use std::cmp;

pub mod tiletypes {
    use euclid::TypedPoint2D;
    pub struct TileSpace;
    pub type TilePoint = TypedPoint2D<u8, TileSpace>;
    pub const TILE_SIZE: usize = 16;
}

use self::tiletypes::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Alpha(pub u8);

impl Alpha {
    const MAX_VALUE: u8 = 0b00001111;
    const BLEND_TABLE: [[u8; 256]; 16] = [
        [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ],
        [
            0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2,
            2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4,
            4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 10,
            10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 11, 11, 11, 11, 11, 11, 11, 11,
            11, 11, 11, 11, 11, 11, 11, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12,
            13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 13, 14, 14, 14, 14, 14, 14, 14,
            14, 14, 14, 14, 14, 14, 14, 14, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
            15, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 17, 17, 17, 17, 17, 17,
            17, 17,
        ],
        [
            0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4,
            4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, 8,
            8, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 10, 10, 10, 10, 10, 10, 10, 11, 11, 11, 11,
            11, 11, 11, 11, 12, 12, 12, 12, 12, 12, 12, 13, 13, 13, 13, 13, 13, 13, 13, 14, 14, 14,
            14, 14, 14, 14, 15, 15, 15, 15, 15, 15, 15, 15, 16, 16, 16, 16, 16, 16, 16, 17, 17, 17,
            17, 17, 17, 17, 17, 18, 18, 18, 18, 18, 18, 18, 19, 19, 19, 19, 19, 19, 19, 19, 20, 20,
            20, 20, 20, 20, 20, 21, 21, 21, 21, 21, 21, 21, 21, 22, 22, 22, 22, 22, 22, 22, 23, 23,
            23, 23, 23, 23, 23, 23, 24, 24, 24, 24, 24, 24, 24, 25, 25, 25, 25, 25, 25, 25, 25, 26,
            26, 26, 26, 26, 26, 26, 27, 27, 27, 27, 27, 27, 27, 27, 28, 28, 28, 28, 28, 28, 28, 29,
            29, 29, 29, 29, 29, 29, 29, 30, 30, 30, 30, 30, 30, 30, 31, 31, 31, 31, 31, 31, 31, 31,
            32, 32, 32, 32, 32, 32, 32, 33, 33, 33, 33, 33, 33, 33, 33, 34, 34, 34, 34,
        ],
        [
            0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 6,
            6, 6, 6, 6, 7, 7, 7, 7, 7, 8, 8, 8, 8, 8, 9, 9, 9, 9, 9, 10, 10, 10, 10, 10, 11, 11,
            11, 11, 11, 12, 12, 12, 12, 12, 13, 13, 13, 13, 13, 14, 14, 14, 14, 14, 15, 15, 15, 15,
            15, 16, 16, 16, 16, 16, 17, 17, 17, 17, 17, 18, 18, 18, 18, 18, 19, 19, 19, 19, 19, 20,
            20, 20, 20, 20, 21, 21, 21, 21, 21, 22, 22, 22, 22, 22, 23, 23, 23, 23, 23, 24, 24, 24,
            24, 24, 25, 25, 25, 25, 25, 26, 26, 26, 26, 26, 27, 27, 27, 27, 27, 28, 28, 28, 28, 28,
            29, 29, 29, 29, 29, 30, 30, 30, 30, 30, 31, 31, 31, 31, 31, 32, 32, 32, 32, 32, 33, 33,
            33, 33, 33, 34, 34, 34, 34, 34, 35, 35, 35, 35, 35, 36, 36, 36, 36, 36, 37, 37, 37, 37,
            37, 38, 38, 38, 38, 38, 39, 39, 39, 39, 39, 40, 40, 40, 40, 40, 41, 41, 41, 41, 41, 42,
            42, 42, 42, 42, 43, 43, 43, 43, 43, 44, 44, 44, 44, 44, 45, 45, 45, 45, 45, 46, 46, 46,
            46, 46, 47, 47, 47, 47, 47, 48, 48, 48, 48, 48, 49, 49, 49, 49, 49, 50, 50, 50, 50, 50,
            51, 51, 51,
        ],
        [
            0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 5, 5, 5, 5, 6, 6, 6, 6, 7, 7, 7, 7,
            8, 8, 8, 9, 9, 9, 9, 10, 10, 10, 10, 11, 11, 11, 11, 12, 12, 12, 13, 13, 13, 13, 14,
            14, 14, 14, 15, 15, 15, 15, 16, 16, 16, 17, 17, 17, 17, 18, 18, 18, 18, 19, 19, 19, 19,
            20, 20, 20, 21, 21, 21, 21, 22, 22, 22, 22, 23, 23, 23, 23, 24, 24, 24, 25, 25, 25, 25,
            26, 26, 26, 26, 27, 27, 27, 27, 28, 28, 28, 29, 29, 29, 29, 30, 30, 30, 30, 31, 31, 31,
            31, 32, 32, 32, 33, 33, 33, 33, 34, 34, 34, 34, 35, 35, 35, 35, 36, 36, 36, 37, 37, 37,
            37, 38, 38, 38, 38, 39, 39, 39, 39, 40, 40, 40, 41, 41, 41, 41, 42, 42, 42, 42, 43, 43,
            43, 43, 44, 44, 44, 45, 45, 45, 45, 46, 46, 46, 46, 47, 47, 47, 47, 48, 48, 48, 49, 49,
            49, 49, 50, 50, 50, 50, 51, 51, 51, 51, 52, 52, 52, 53, 53, 53, 53, 54, 54, 54, 54, 55,
            55, 55, 55, 56, 56, 56, 57, 57, 57, 57, 58, 58, 58, 58, 59, 59, 59, 59, 60, 60, 60, 61,
            61, 61, 61, 62, 62, 62, 62, 63, 63, 63, 63, 64, 64, 64, 65, 65, 65, 65, 66, 66, 66, 66,
            67, 67, 67, 67, 68, 68,
        ],
        [
            0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 6, 7, 7, 7, 8, 8, 8, 9, 9, 9,
            10, 10, 10, 11, 11, 11, 12, 12, 12, 13, 13, 13, 14, 14, 14, 15, 15, 15, 16, 16, 16, 17,
            17, 17, 18, 18, 18, 19, 19, 19, 20, 20, 20, 21, 21, 21, 22, 22, 22, 23, 23, 23, 24, 24,
            24, 25, 25, 25, 26, 26, 26, 27, 27, 27, 28, 28, 28, 29, 29, 29, 30, 30, 30, 31, 31, 31,
            32, 32, 32, 33, 33, 33, 34, 34, 34, 35, 35, 35, 36, 36, 36, 37, 37, 37, 38, 38, 38, 39,
            39, 39, 40, 40, 40, 41, 41, 41, 42, 42, 42, 43, 43, 43, 44, 44, 44, 45, 45, 45, 46, 46,
            46, 47, 47, 47, 48, 48, 48, 49, 49, 49, 50, 50, 50, 51, 51, 51, 52, 52, 52, 53, 53, 53,
            54, 54, 54, 55, 55, 55, 56, 56, 56, 57, 57, 57, 58, 58, 58, 59, 59, 59, 60, 60, 60, 61,
            61, 61, 62, 62, 62, 63, 63, 63, 64, 64, 64, 65, 65, 65, 66, 66, 66, 67, 67, 67, 68, 68,
            68, 69, 69, 69, 70, 70, 70, 71, 71, 71, 72, 72, 72, 73, 73, 73, 74, 74, 74, 75, 75, 75,
            76, 76, 76, 77, 77, 77, 78, 78, 78, 79, 79, 79, 80, 80, 80, 81, 81, 81, 82, 82, 82, 83,
            83, 83, 84, 84, 84, 85, 85,
        ],
        [
            0, 0, 1, 1, 2, 2, 2, 3, 3, 4, 4, 4, 5, 5, 6, 6, 6, 7, 7, 8, 8, 8, 9, 9, 10, 10, 10, 11,
            11, 12, 12, 12, 13, 13, 14, 14, 14, 15, 15, 16, 16, 16, 17, 17, 18, 18, 18, 19, 19, 20,
            20, 20, 21, 21, 22, 22, 22, 23, 23, 24, 24, 24, 25, 25, 26, 26, 26, 27, 27, 28, 28, 28,
            29, 29, 30, 30, 30, 31, 31, 32, 32, 32, 33, 33, 34, 34, 34, 35, 35, 36, 36, 36, 37, 37,
            38, 38, 38, 39, 39, 40, 40, 40, 41, 41, 42, 42, 42, 43, 43, 44, 44, 44, 45, 45, 46, 46,
            46, 47, 47, 48, 48, 48, 49, 49, 50, 50, 50, 51, 51, 52, 52, 52, 53, 53, 54, 54, 54, 55,
            55, 56, 56, 56, 57, 57, 58, 58, 58, 59, 59, 60, 60, 60, 61, 61, 62, 62, 62, 63, 63, 64,
            64, 64, 65, 65, 66, 66, 66, 67, 67, 68, 68, 68, 69, 69, 70, 70, 70, 71, 71, 72, 72, 72,
            73, 73, 74, 74, 74, 75, 75, 76, 76, 76, 77, 77, 78, 78, 78, 79, 79, 80, 80, 80, 81, 81,
            82, 82, 82, 83, 83, 84, 84, 84, 85, 85, 86, 86, 86, 87, 87, 88, 88, 88, 89, 89, 90, 90,
            90, 91, 91, 92, 92, 92, 93, 93, 94, 94, 94, 95, 95, 96, 96, 96, 97, 97, 98, 98, 98, 99,
            99, 100, 100, 100, 101, 101, 102, 102,
        ],
        [
            0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12,
            13, 13, 14, 14, 14, 15, 15, 16, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 21, 22, 22,
            23, 23, 24, 24, 25, 25, 26, 26, 27, 27, 28, 28, 28, 29, 29, 30, 30, 31, 31, 32, 32, 33,
            33, 34, 34, 35, 35, 35, 36, 36, 37, 37, 38, 38, 39, 39, 40, 40, 41, 41, 42, 42, 42, 43,
            43, 44, 44, 45, 45, 46, 46, 47, 47, 48, 48, 49, 49, 49, 50, 50, 51, 51, 52, 52, 53, 53,
            54, 54, 55, 55, 56, 56, 56, 57, 57, 58, 58, 59, 59, 60, 60, 61, 61, 62, 62, 63, 63, 63,
            64, 64, 65, 65, 66, 66, 67, 67, 68, 68, 69, 69, 70, 70, 70, 71, 71, 72, 72, 73, 73, 74,
            74, 75, 75, 76, 76, 77, 77, 77, 78, 78, 79, 79, 80, 80, 81, 81, 82, 82, 83, 83, 84, 84,
            84, 85, 85, 86, 86, 87, 87, 88, 88, 89, 89, 90, 90, 91, 91, 91, 92, 92, 93, 93, 94, 94,
            95, 95, 96, 96, 97, 97, 98, 98, 98, 99, 99, 100, 100, 101, 101, 102, 102, 103, 103,
            104, 104, 105, 105, 105, 106, 106, 107, 107, 108, 108, 109, 109, 110, 110, 111, 111,
            112, 112, 112, 113, 113, 114, 114, 115, 115, 116, 116, 117, 117, 118, 118, 119, 119,
        ],
        [
            0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
            14, 14, 15, 15, 16, 17, 17, 18, 18, 19, 19, 20, 20, 21, 21, 22, 22, 23, 23, 24, 25, 25,
            26, 26, 27, 27, 28, 28, 29, 29, 30, 30, 31, 31, 32, 33, 33, 34, 34, 35, 35, 36, 36, 37,
            37, 38, 38, 39, 39, 40, 41, 41, 42, 42, 43, 43, 44, 44, 45, 45, 46, 46, 47, 47, 48, 49,
            49, 50, 50, 51, 51, 52, 52, 53, 53, 54, 54, 55, 55, 56, 57, 57, 58, 58, 59, 59, 60, 60,
            61, 61, 62, 62, 63, 63, 64, 65, 65, 66, 66, 67, 67, 68, 68, 69, 69, 70, 70, 71, 71, 72,
            73, 73, 74, 74, 75, 75, 76, 76, 77, 77, 78, 78, 79, 79, 80, 81, 81, 82, 82, 83, 83, 84,
            84, 85, 85, 86, 86, 87, 87, 88, 89, 89, 90, 90, 91, 91, 92, 92, 93, 93, 94, 94, 95, 95,
            96, 97, 97, 98, 98, 99, 99, 100, 100, 101, 101, 102, 102, 103, 103, 104, 105, 105, 106,
            106, 107, 107, 108, 108, 109, 109, 110, 110, 111, 111, 112, 113, 113, 114, 114, 115,
            115, 116, 116, 117, 117, 118, 118, 119, 119, 120, 121, 121, 122, 122, 123, 123, 124,
            124, 125, 125, 126, 126, 127, 127, 128, 129, 129, 130, 130, 131, 131, 132, 132, 133,
            133, 134, 134, 135, 135, 136,
        ],
        [
            0, 1, 1, 2, 2, 3, 4, 4, 5, 5, 6, 7, 7, 8, 8, 9, 10, 10, 11, 11, 12, 13, 13, 14, 14, 15,
            16, 16, 17, 17, 18, 19, 19, 20, 20, 21, 22, 22, 23, 23, 24, 25, 25, 26, 26, 27, 28, 28,
            29, 29, 30, 31, 31, 32, 32, 33, 34, 34, 35, 35, 36, 37, 37, 38, 38, 39, 40, 40, 41, 41,
            42, 43, 43, 44, 44, 45, 46, 46, 47, 47, 48, 49, 49, 50, 50, 51, 52, 52, 53, 53, 54, 55,
            55, 56, 56, 57, 58, 58, 59, 59, 60, 61, 61, 62, 62, 63, 64, 64, 65, 65, 66, 67, 67, 68,
            68, 69, 70, 70, 71, 71, 72, 73, 73, 74, 74, 75, 76, 76, 77, 77, 78, 79, 79, 80, 80, 81,
            82, 82, 83, 83, 84, 85, 85, 86, 86, 87, 88, 88, 89, 89, 90, 91, 91, 92, 92, 93, 94, 94,
            95, 95, 96, 97, 97, 98, 98, 99, 100, 100, 101, 101, 102, 103, 103, 104, 104, 105, 106,
            106, 107, 107, 108, 109, 109, 110, 110, 111, 112, 112, 113, 113, 114, 115, 115, 116,
            116, 117, 118, 118, 119, 119, 120, 121, 121, 122, 122, 123, 124, 124, 125, 125, 126,
            127, 127, 128, 128, 129, 130, 130, 131, 131, 132, 133, 133, 134, 134, 135, 136, 136,
            137, 137, 138, 139, 139, 140, 140, 141, 142, 142, 143, 143, 144, 145, 145, 146, 146,
            147, 148, 148, 149, 149, 150, 151, 151, 152, 152, 153,
        ],
        [
            0, 1, 1, 2, 3, 3, 4, 5, 5, 6, 7, 7, 8, 9, 9, 10, 11, 11, 12, 13, 13, 14, 15, 15, 16,
            17, 17, 18, 19, 19, 20, 21, 21, 22, 23, 23, 24, 25, 25, 26, 27, 27, 28, 29, 29, 30, 31,
            31, 32, 33, 33, 34, 35, 35, 36, 37, 37, 38, 39, 39, 40, 41, 41, 42, 43, 43, 44, 45, 45,
            46, 47, 47, 48, 49, 49, 50, 51, 51, 52, 53, 53, 54, 55, 55, 56, 57, 57, 58, 59, 59, 60,
            61, 61, 62, 63, 63, 64, 65, 65, 66, 67, 67, 68, 69, 69, 70, 71, 71, 72, 73, 73, 74, 75,
            75, 76, 77, 77, 78, 79, 79, 80, 81, 81, 82, 83, 83, 84, 85, 85, 86, 87, 87, 88, 89, 89,
            90, 91, 91, 92, 93, 93, 94, 95, 95, 96, 97, 97, 98, 99, 99, 100, 101, 101, 102, 103,
            103, 104, 105, 105, 106, 107, 107, 108, 109, 109, 110, 111, 111, 112, 113, 113, 114,
            115, 115, 116, 117, 117, 118, 119, 119, 120, 121, 121, 122, 123, 123, 124, 125, 125,
            126, 127, 127, 128, 129, 129, 130, 131, 131, 132, 133, 133, 134, 135, 135, 136, 137,
            137, 138, 139, 139, 140, 141, 141, 142, 143, 143, 144, 145, 145, 146, 147, 147, 148,
            149, 149, 150, 151, 151, 152, 153, 153, 154, 155, 155, 156, 157, 157, 158, 159, 159,
            160, 161, 161, 162, 163, 163, 164, 165, 165, 166, 167, 167, 168, 169, 169, 170,
        ],
        [
            0, 1, 1, 2, 3, 4, 4, 5, 6, 7, 7, 8, 9, 10, 10, 11, 12, 12, 13, 14, 15, 15, 16, 17, 18,
            18, 19, 20, 21, 21, 22, 23, 23, 24, 25, 26, 26, 27, 28, 29, 29, 30, 31, 32, 32, 33, 34,
            34, 35, 36, 37, 37, 38, 39, 40, 40, 41, 42, 43, 43, 44, 45, 45, 46, 47, 48, 48, 49, 50,
            51, 51, 52, 53, 54, 54, 55, 56, 56, 57, 58, 59, 59, 60, 61, 62, 62, 63, 64, 65, 65, 66,
            67, 67, 68, 69, 70, 70, 71, 72, 73, 73, 74, 75, 76, 76, 77, 78, 78, 79, 80, 81, 81, 82,
            83, 84, 84, 85, 86, 87, 87, 88, 89, 89, 90, 91, 92, 92, 93, 94, 95, 95, 96, 97, 98, 98,
            99, 100, 100, 101, 102, 103, 103, 104, 105, 106, 106, 107, 108, 109, 109, 110, 111,
            111, 112, 113, 114, 114, 115, 116, 117, 117, 118, 119, 120, 120, 121, 122, 122, 123,
            124, 125, 125, 126, 127, 128, 128, 129, 130, 131, 131, 132, 133, 133, 134, 135, 136,
            136, 137, 138, 139, 139, 140, 141, 142, 142, 143, 144, 144, 145, 146, 147, 147, 148,
            149, 150, 150, 151, 152, 153, 153, 154, 155, 155, 156, 157, 158, 158, 159, 160, 161,
            161, 162, 163, 164, 164, 165, 166, 166, 167, 168, 169, 169, 170, 171, 172, 172, 173,
            174, 175, 175, 176, 177, 177, 178, 179, 180, 180, 181, 182, 183, 183, 184, 185, 186,
            186, 187,
        ],
        [
            0, 1, 2, 2, 3, 4, 5, 6, 6, 7, 8, 9, 10, 10, 11, 12, 13, 14, 14, 15, 16, 17, 18, 18, 19,
            20, 21, 22, 22, 23, 24, 25, 26, 26, 27, 28, 29, 30, 30, 31, 32, 33, 34, 34, 35, 36, 37,
            38, 38, 39, 40, 41, 42, 42, 43, 44, 45, 46, 46, 47, 48, 49, 50, 50, 51, 52, 53, 54, 54,
            55, 56, 57, 58, 58, 59, 60, 61, 62, 62, 63, 64, 65, 66, 66, 67, 68, 69, 70, 70, 71, 72,
            73, 74, 74, 75, 76, 77, 78, 78, 79, 80, 81, 82, 82, 83, 84, 85, 86, 86, 87, 88, 89, 90,
            90, 91, 92, 93, 94, 94, 95, 96, 97, 98, 98, 99, 100, 101, 102, 102, 103, 104, 105, 106,
            106, 107, 108, 109, 110, 110, 111, 112, 113, 114, 114, 115, 116, 117, 118, 118, 119,
            120, 121, 122, 122, 123, 124, 125, 126, 126, 127, 128, 129, 130, 130, 131, 132, 133,
            134, 134, 135, 136, 137, 138, 138, 139, 140, 141, 142, 142, 143, 144, 145, 146, 146,
            147, 148, 149, 150, 150, 151, 152, 153, 154, 154, 155, 156, 157, 158, 158, 159, 160,
            161, 162, 162, 163, 164, 165, 166, 166, 167, 168, 169, 170, 170, 171, 172, 173, 174,
            174, 175, 176, 177, 178, 178, 179, 180, 181, 182, 182, 183, 184, 185, 186, 186, 187,
            188, 189, 190, 190, 191, 192, 193, 194, 194, 195, 196, 197, 198, 198, 199, 200, 201,
            202, 202, 203, 204,
        ],
        [
            0, 1, 2, 3, 3, 4, 5, 6, 7, 8, 9, 10, 10, 11, 12, 13, 14, 15, 16, 16, 17, 18, 19, 20,
            21, 22, 23, 23, 24, 25, 26, 27, 28, 29, 29, 30, 31, 32, 33, 34, 35, 36, 36, 37, 38, 39,
            40, 41, 42, 42, 43, 44, 45, 46, 47, 48, 49, 49, 50, 51, 52, 53, 54, 55, 55, 56, 57, 58,
            59, 60, 61, 62, 62, 63, 64, 65, 66, 67, 68, 68, 69, 70, 71, 72, 73, 74, 75, 75, 76, 77,
            78, 79, 80, 81, 81, 82, 83, 84, 85, 86, 87, 88, 88, 89, 90, 91, 92, 93, 94, 94, 95, 96,
            97, 98, 99, 100, 101, 101, 102, 103, 104, 105, 106, 107, 107, 108, 109, 110, 111, 112,
            113, 114, 114, 115, 116, 117, 118, 119, 120, 120, 121, 122, 123, 124, 125, 126, 127,
            127, 128, 129, 130, 131, 132, 133, 133, 134, 135, 136, 137, 138, 139, 140, 140, 141,
            142, 143, 144, 145, 146, 146, 147, 148, 149, 150, 151, 152, 153, 153, 154, 155, 156,
            157, 158, 159, 159, 160, 161, 162, 163, 164, 165, 166, 166, 167, 168, 169, 170, 171,
            172, 172, 173, 174, 175, 176, 177, 178, 179, 179, 180, 181, 182, 183, 184, 185, 185,
            186, 187, 188, 189, 190, 191, 192, 192, 193, 194, 195, 196, 197, 198, 198, 199, 200,
            201, 202, 203, 204, 205, 205, 206, 207, 208, 209, 210, 211, 211, 212, 213, 214, 215,
            216, 217, 218, 218, 219, 220, 221,
        ],
        [
            0, 1, 2, 3, 4, 5, 6, 7, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 21,
            22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 35, 36, 37, 38, 39, 40, 41, 42,
            43, 44, 45, 46, 47, 48, 49, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63,
            63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 77, 78, 79, 80, 81, 82, 83,
            84, 85, 86, 87, 88, 89, 90, 91, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103,
            104, 105, 105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119,
            119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 133, 134,
            135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 147, 148, 149, 150,
            151, 152, 153, 154, 155, 156, 157, 158, 159, 160, 161, 161, 162, 163, 164, 165, 166,
            167, 168, 169, 170, 171, 172, 173, 174, 175, 175, 176, 177, 178, 179, 180, 181, 182,
            183, 184, 185, 186, 187, 188, 189, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198,
            199, 200, 201, 202, 203, 203, 204, 205, 206, 207, 208, 209, 210, 211, 212, 213, 214,
            215, 216, 217, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227, 228, 229, 230,
            231, 231, 232, 233, 234, 235, 236, 237, 238,
        ],
        [
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
            46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67,
            68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89,
            90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108,
            109, 110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125,
            126, 127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142,
            143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159,
            160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 174, 175, 176,
            177, 178, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191, 192, 193,
            194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 210,
            211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 223, 224, 225, 226, 227,
            228, 229, 230, 231, 232, 233, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244,
            245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255,
        ],
    ];
    pub fn max(&mut self, other: Self) -> &mut Self {
        self.0 = cmp::max(self.0, other.0);
        self
    }
    fn inv(self) -> Alpha {
        Alpha(Self::MAX_VALUE - self.0)
    }
    fn is_trans(self) -> bool {
        self.0 == 0
    }
    fn blend(self, orig: u8, new: u8) -> u8 {
        let o = self.inv().comp(orig);
        let n = self.comp(new);
        o + n
    }
    fn comp(self, v: u8) -> u8 {
        let alpha = usize::from(self.0);
        Self::BLEND_TABLE[alpha][usize::from(v)]
    }
    pub fn plus(&mut self, u: u8) -> &mut Self {
        self.0 = cmp::min(self.0 + u, Self::MAX_VALUE);
        self
    }
    pub fn from_f32(f: f32) -> Alpha {
        let v = cmp::min((f * 15.0).ceil() as u8, Self::MAX_VALUE);
        Alpha(v)
    }
    pub fn to_256(&self) -> u8 {
        (0..4).fold(0u8, |res, i| {
            let bit = self.0 & (1 << i);
            res | bit << (i + 1)
        })
    }
}

/// In altena16 Alpha Value has special meaning
/// 0b1011
/// the first 4 digit represents the Alpha value
/// the last 4 digits are used to customize collison attribute
///       1010
pub trait AltenaAlpha {
    const ALPHA_MASK: u8 = 0b11110000;
    const COLLISION_MASK: u8 = 0b00001111;
    fn value(&self) -> u8;
    fn collision_bits(&self) -> u8 {
        self.value() & Self::COLLISION_MASK
    }
    fn alpha(&self) -> Alpha {
        Alpha((self.value() & Self::ALPHA_MASK) >> 4)
    }
    fn is_trans(&self) -> bool {
        self.alpha().is_trans()
    }
}

impl<T: Primitive> AltenaAlpha for Rgba<T> {
    fn value(&self) -> u8 {
        match self[3].to_u8() {
            Some(a) => a,
            None => 0,
        }
    }
}

/// altena don't support alpha blending, so just rgb is enough
#[derive(Clone, Copy, Debug, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn from_rgba<T: Primitive>(rgba: &Rgba<T>) -> Option<Color> {
        if rgba.is_trans() {
            return None;
        }
        Some(Color {
            r: rgba[0].to_u8()?,
            g: rgba[1].to_u8()?,
            b: rgba[2].to_u8()?,
        })
    }
    pub fn to_rgba<T: Primitive + From<u8>>(&self) -> Rgba<T> {
        let mut res = Rgba {
            data: [T::zero(); 4],
        };
        res[0] = convert::From::from(self.r);
        res[1] = convert::From::from(self.g);
        res[2] = convert::From::from(self.b);
        res
    }
    pub fn to_term(&self) -> TermRGB {
        TermRGB::RGB(self.r, self.g, self.b)
    }
    pub fn black() -> Self {
        Color { r: 0, g: 0, b: 0 }
    }
    pub fn white() -> Self {
        Color {
            r: 255,
            g: 255,
            b: 255,
        }
    }
    fn get(&self, id: usize) -> Option<u8> {
        match id {
            0 => Some(self.r),
            1 => Some(self.g),
            2 => Some(self.b),
            _ => None,
        }
    }
}

pub trait Blend {
    fn blend(&mut self, other: Color, alpha: Alpha);
}

impl Blend for Color {
    fn blend(&mut self, other: Color, alpha: Alpha) {
        self.r = alpha.blend(self.r, other.r);
        self.g = alpha.blend(self.g, other.g);
        self.b = alpha.blend(self.b, other.b);
    }
}

impl<T: Primitive> Blend for Rgba<T> {
    fn blend(&mut self, other: Color, alpha: Alpha) {
        (0..3)
            .try_for_each(|i| {
                let v = alpha.blend(self[i].to_u8()?, other.get(i)?);
                self[i] = T::from(v)?;
                Some(())
            })
            .unwrap();
    }
}

impl Blend for Dot {
    fn blend(&mut self, other: Color, alpha: Alpha) {
        match *self {
            Some(mut rgb) => {
                rgb.blend(other, alpha);
                *self = Some(rgb);
            }
            None => {
                let mut new = Color::default();
                new.r = alpha.inv().comp(other.r);
                new.g = alpha.inv().comp(other.g);
                new.b = alpha.inv().comp(other.b);
                *self = Some(new)
            }
        }
    }
}

pub trait Draw {
    fn blend(&mut self, other: Color, alpha: Alpha);
}

pub type Dot = Option<Color>;

fn dot_fmt(d: &Dot, f: &mut fmt::Formatter) -> fmt::Result {
    match d {
        Some(rgb) => write!(f, "{}", Style::new().on(rgb.to_term()).paint("  ")),
        None => write!(f, "  "),
    }
}

/// 16×16 tile used to draw objects.
#[derive(Clone)]
pub struct Tile {
    /// Buffer of tile data
    inner: [Dot; TILE_SIZE * TILE_SIZE],
}

impl fmt::Debug for Tile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "tile: {{")?;
        for i in 0..TILE_SIZE {
            for j in 0..TILE_SIZE {
                let dot = self.get_xy(j, i).unwrap();
                dot_fmt(&dot, f)?;
            }
            writeln!(f, "")?;
        }
        writeln!(f, "}}")
    }
}

impl Default for Tile {
    fn default() -> Tile {
        Tile {
            inner: [None; TILE_SIZE * TILE_SIZE],
        }
    }
}

impl Tile {
    pub fn new(d: Dot) -> Tile {
        Tile {
            inner: [d; TILE_SIZE * TILE_SIZE],
        }
    }
}

impl Get2D for Tile {
    type Item = Dot;
    fn get_xy<T: ToPrimitive>(&self, x: T, y: T) -> Option<&Dot> {
        let (x, y) = (x.to_usize()?, y.to_usize()?);
        if TILE_SIZE <= x || TILE_SIZE <= y {
            return None;
        }
        Some(&self.inner[y * TILE_SIZE + x])
    }
}

impl GetMut2D for Tile {
    type Item = Dot;
    fn get_mut_xy<T: ToPrimitive>(&mut self, x: T, y: T) -> Option<&mut Dot> {
        let (x, y) = (x.to_usize()?, y.to_usize()?);
        if TILE_SIZE <= x || TILE_SIZE <= y {
            return None;
        }
        Some(&mut self.inner[y * TILE_SIZE + x])
    }
}
