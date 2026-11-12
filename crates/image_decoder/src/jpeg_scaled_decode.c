#include <setjmp.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include <jpeglib.h>

struct rechrom_jpeg_error {
  struct jpeg_error_mgr base;
  jmp_buf jump;
  char message[JMSG_LENGTH_MAX];
};

static void rechrom_jpeg_error_exit(j_common_ptr common) {
  struct rechrom_jpeg_error* error =
      (struct rechrom_jpeg_error*)common->err;
  (*common->err->format_message)(common, error->message);
  longjmp(error->jump, 1);
}

// Returns 0 on success, 1 when this encoded image cannot produce the exact
// requested libjpeg scaled size, and -1 on malformed input/decode failure.
int rechrom_jpeg_decode_scaled_n32(const uint8_t* encoded,
                                   size_t encoded_length,
                                   int target_width,
                                   int target_height,
                                   uint8_t* destination,
                                   size_t destination_length,
                                   char* error_message,
                                   size_t error_message_length) {
  struct jpeg_decompress_struct decoder;
  struct rechrom_jpeg_error error;
  volatile int created = 0;
  memset(&decoder, 0, sizeof(decoder));
  memset(&error, 0, sizeof(error));
  decoder.err = jpeg_std_error(&error.base);
  error.base.error_exit = rechrom_jpeg_error_exit;
  if (setjmp(error.jump)) {
    if (created) {
      jpeg_destroy_decompress(&decoder);
    }
    if (error_message && error_message_length) {
      snprintf(error_message, error_message_length, "%s", error.message);
    }
    return -1;
  }

  jpeg_create_decompress(&decoder);
  created = 1;
  jpeg_mem_src(&decoder, encoded, (unsigned long)encoded_length);
  if (jpeg_read_header(&decoder, TRUE) != JPEG_HEADER_OK) {
    jpeg_destroy_decompress(&decoder);
    return -1;
  }
  switch (decoder.jpeg_color_space) {
    case JCS_YCbCr:
    case JCS_GRAYSCALE:
    case JCS_RGB:
      break;
    default:
      jpeg_destroy_decompress(&decoder);
      return 1;
  }

  // Source: Blink JPEGImageReader::ShouldDecodeToOriginalSize(). Chromium
  // deliberately withholds reduced decode sizes when the encoded dimensions
  // do not contain a whole number of MCUs. libjpeg-turbo can otherwise expose
  // padding from the final MCU row or column while downscaling (crbug.com/890745).
  // Desktop Chromium has no decoded-image byte limit, so only the original
  // size remains supported for such images.
  if ((target_width != (int)decoder.image_width ||
       target_height != (int)decoder.image_height)) {
    const int max_h = decoder.max_h_samp_factor;
    const int max_v = decoder.max_v_samp_factor;
    if (max_h < 1 || max_h > 4 || max_v < 1 || max_v > 4) {
      jpeg_destroy_decompress(&decoder);
      return 1;
    }
    const int mcu_width = max_h * DCTSIZE;
    const int mcu_height = max_v * DCTSIZE;
    if (decoder.image_width % mcu_width != 0 ||
        decoder.image_height % mcu_height != 0) {
      jpeg_destroy_decompress(&decoder);
      return 1;
    }
  }

  int numerator = 0;
  decoder.scale_denom = 8;
  for (int candidate = 1; candidate <= 8; ++candidate) {
    decoder.scale_num = candidate;
    jpeg_calc_output_dimensions(&decoder);
    if ((int)decoder.output_width == target_width &&
        (int)decoder.output_height == target_height) {
      numerator = candidate;
      break;
    }
  }
  if (!numerator) {
    jpeg_destroy_decompress(&decoder);
    return 1;
  }
  decoder.scale_num = numerator;
#if defined(JCS_ALPHA_EXTENSIONS)
#if defined(RECHROM_N32_BGRA)
  decoder.out_color_space = JCS_EXT_BGRA;
#else
  decoder.out_color_space = JCS_EXT_RGBA;
#endif
#else
  jpeg_destroy_decompress(&decoder);
  return 1;
#endif
  jpeg_calc_output_dimensions(&decoder);
  const size_t row_bytes = (size_t)target_width * 4;
  if (destination_length < row_bytes * (size_t)target_height) {
    jpeg_destroy_decompress(&decoder);
    return -1;
  }
  if (!jpeg_start_decompress(&decoder) || decoder.output_components != 4 ||
      (int)decoder.output_width != target_width ||
      (int)decoder.output_height != target_height) {
    jpeg_destroy_decompress(&decoder);
    return -1;
  }
  while (decoder.output_scanline < decoder.output_height) {
    JSAMPROW row = destination + (size_t)decoder.output_scanline * row_bytes;
    if (jpeg_read_scanlines(&decoder, &row, 1) != 1) {
      jpeg_destroy_decompress(&decoder);
      return -1;
    }
  }
  if (!jpeg_finish_decompress(&decoder)) {
    jpeg_destroy_decompress(&decoder);
    return -1;
  }
  jpeg_destroy_decompress(&decoder);
  return 0;
}
