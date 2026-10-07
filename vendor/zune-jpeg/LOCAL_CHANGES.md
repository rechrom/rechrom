Local changes to zune-jpeg 0.5.15
================================
The opt-in source_compat feature aligns 8-bit JPEG decoding with the C++
SkiaImageDecoder's libjpeg-turbo JDCT_ISLOW and fancy-upsample configuration.
The Rust kernels adapt integer arithmetic from libjpeg-turbo jidctint.c,
jdcolor.c, and jdsample.c. These modified kernels are not original libjpeg-turbo
files. Copyright notices and IJG terms are retained in their headers and in
source-compat-licenses/. This software is based in part on the work of the
Independent JPEG Group.

The existing zune parser, entropy decoder, public API and default kernels remain.
The compatibility feature selects 13-bit ISLOW for both full and sparse blocks,
16-bit YCbCr conversion, coupled two-axis fancy upsampling, and replication of
the actual final component row instead of a padded MCU row. No native decoder is
called. The default feature configuration keeps upstream dispatch behavior.
