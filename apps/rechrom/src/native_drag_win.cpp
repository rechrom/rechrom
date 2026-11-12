// Native OLE source adapter. Chromium counterparts: drag_drop_client_win.cc,
// os_exchange_data_provider_win.cc and data_object_impl.cc. The document's
// interaction and DataTransfer owners never enter this native nested loop.
#define NOMINMAX
#define UNICODE
#define _UNICODE
#include <windows.h>
#include <ole2.h>
#include <shlobj.h>
#include <shlwapi.h>
#include <algorithm>
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

struct RechromDragString { const wchar_t* mime; size_t mime_len; const wchar_t* text; size_t len; };
struct RechromDragFile { const wchar_t* name; size_t name_len; const wchar_t* path; size_t path_len; const unsigned char* bytes; size_t len; };
struct RechromDragImage { const unsigned char* rgba; unsigned width; unsigned height; size_t stride; };

namespace {
HGLOBAL CopyGlobal(const void* bytes, size_t size) {
  HGLOBAL handle = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, std::max<size_t>(size, 1));
  if (!handle) return nullptr;
  void* destination = GlobalLock(handle);
  if (!destination) { GlobalFree(handle); return nullptr; }
  if (size) memcpy(destination, bytes, size);
  GlobalUnlock(handle);
  return handle;
}
struct Entry {
  FORMATETC format;
  std::vector<unsigned char> bytes;
  std::wstring path;
};
class FormatEnum final : public IEnumFORMATETC {
 public:
  explicit FormatEnum(std::vector<FORMATETC> formats, size_t position = 0)
      : formats_(std::move(formats)), position_(position) {}
  HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** result) override {
    if (!result) return E_POINTER;
    *result = nullptr;
    if (iid == IID_IUnknown || iid == IID_IEnumFORMATETC) { *result = this; AddRef(); return S_OK; }
    return E_NOINTERFACE;
  }
  ULONG STDMETHODCALLTYPE AddRef() override { return InterlockedIncrement(&refs_); }
  ULONG STDMETHODCALLTYPE Release() override { ULONG n = InterlockedDecrement(&refs_); if (!n) delete this; return n; }
  HRESULT STDMETHODCALLTYPE Next(ULONG count, FORMATETC* output, ULONG* fetched) override {
    if (!output || (!fetched && count != 1)) return E_POINTER;
    ULONG n = 0;
    while (n < count && position_ < formats_.size()) output[n++] = formats_[position_++];
    if (fetched) *fetched = n;
    return n == count ? S_OK : S_FALSE;
  }
  HRESULT STDMETHODCALLTYPE Skip(ULONG count) override {
    size_t n = std::min<size_t>(count, formats_.size() - position_); position_ += n;
    return n == count ? S_OK : S_FALSE;
  }
  HRESULT STDMETHODCALLTYPE Reset() override { position_ = 0; return S_OK; }
  HRESULT STDMETHODCALLTYPE Clone(IEnumFORMATETC** result) override {
    if (!result) return E_POINTER;
    *result = new FormatEnum(formats_, position_); return S_OK;
  }
 private:
  LONG refs_ = 1;
  std::vector<FORMATETC> formats_;
  size_t position_;
};
class Data final : public IDataObject {
 public:
  void Add(CLIPFORMAT format, const void* bytes, size_t size, LONG index = -1, DWORD medium = TYMED_HGLOBAL) {
    Entry entry{{format, nullptr, DVASPECT_CONTENT, index, medium}, {}, {}};
    const auto* begin = static_cast<const unsigned char*>(bytes);
    if (size) entry.bytes.assign(begin, begin + size);
    entries_.push_back(std::move(entry));
  }
  void AddPath(CLIPFORMAT format, std::wstring path, LONG index) {
    entries_.push_back({{format, nullptr, DVASPECT_CONTENT, index, TYMED_ISTREAM}, {}, std::move(path)});
  }
  void AddWide(CLIPFORMAT format, std::wstring text) { text.push_back(0); Add(format, text.data(), text.size() * sizeof(wchar_t)); }
  HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** result) override {
    if (!result) return E_POINTER;
    *result = nullptr;
    if (iid == IID_IUnknown || iid == IID_IDataObject) { *result = this; AddRef(); return S_OK; }
    return E_NOINTERFACE;
  }
  ULONG STDMETHODCALLTYPE AddRef() override { return InterlockedIncrement(&refs_); }
  ULONG STDMETHODCALLTYPE Release() override { ULONG n = InterlockedDecrement(&refs_); if (!n) delete this; return n; }
  HRESULT STDMETHODCALLTYPE GetData(FORMATETC* format, STGMEDIUM* medium) override {
    if (!format || !medium) return E_POINTER;
    *medium = {};
    for (const auto& entry : entries_) {
      if (!Matches(*format, entry.format)) continue;
      if (entry.format.tymed == TYMED_ISTREAM) {
        IStream* stream = nullptr;
        HRESULT status;
        if (!entry.path.empty()) {
          status = SHCreateStreamOnFileEx(entry.path.c_str(), STGM_READ | STGM_SHARE_DENY_NONE, FILE_ATTRIBUTE_NORMAL, FALSE, nullptr, &stream);
        } else {
          HGLOBAL buffer = CopyGlobal(entry.bytes.data(), entry.bytes.size());
          if (!buffer) return E_OUTOFMEMORY;
          status = CreateStreamOnHGlobal(buffer, TRUE, &stream);
          if (FAILED(status)) GlobalFree(buffer);
          if (SUCCEEDED(status)) { ULARGE_INTEGER length; length.QuadPart = entry.bytes.size(); stream->SetSize(length); }
        }
        if (FAILED(status)) return status;
        medium->tymed = TYMED_ISTREAM; medium->pstm = stream;
      } else {
        medium->hGlobal = CopyGlobal(entry.bytes.data(), entry.bytes.size());
        if (!medium->hGlobal) return E_OUTOFMEMORY;
        medium->tymed = TYMED_HGLOBAL;
      }
      return S_OK;
    }
    return DV_E_FORMATETC;
  }
  HRESULT STDMETHODCALLTYPE GetDataHere(FORMATETC*, STGMEDIUM*) override { return E_NOTIMPL; }
  HRESULT STDMETHODCALLTYPE QueryGetData(FORMATETC* format) override {
    if (!format) return E_POINTER;
    for (const auto& entry : entries_) if (Matches(*format, entry.format)) return S_OK;
    return DV_E_FORMATETC;
  }
  HRESULT STDMETHODCALLTYPE GetCanonicalFormatEtc(FORMATETC*, FORMATETC* output) override { if (output) output->ptd = nullptr; return E_NOTIMPL; }
  HRESULT STDMETHODCALLTYPE SetData(FORMATETC* format, STGMEDIUM* medium, BOOL release) override {
    // IDragSourceHelper attaches private drag image formats through SetData.
    if (!format || !medium) return E_POINTER;
    if (medium->tymed != TYMED_HGLOBAL) return DV_E_TYMED;
    void* bytes = GlobalLock(medium->hGlobal);
    if (!bytes) return E_OUTOFMEMORY;
    Add(format->cfFormat, bytes, GlobalSize(medium->hGlobal), format->lindex);
    GlobalUnlock(medium->hGlobal);
    if (release) ReleaseStgMedium(medium);
    return S_OK;
  }
  HRESULT STDMETHODCALLTYPE EnumFormatEtc(DWORD direction, IEnumFORMATETC** result) override {
    if (!result) return E_POINTER;
    *result = nullptr;
    if (direction != DATADIR_GET) return E_NOTIMPL;
    std::vector<FORMATETC> formats;
    for (const auto& entry : entries_) formats.push_back(entry.format);
    *result = new FormatEnum(std::move(formats)); return S_OK;
  }
  HRESULT STDMETHODCALLTYPE DAdvise(FORMATETC*, DWORD, IAdviseSink*, DWORD*) override { return OLE_E_ADVISENOTSUPPORTED; }
  HRESULT STDMETHODCALLTYPE DUnadvise(DWORD) override { return OLE_E_ADVISENOTSUPPORTED; }
  HRESULT STDMETHODCALLTYPE EnumDAdvise(IEnumSTATDATA**) override { return OLE_E_ADVISENOTSUPPORTED; }
 private:
  static bool Matches(const FORMATETC& request, const FORMATETC& stored) {
    return request.cfFormat == stored.cfFormat && request.dwAspect == DVASPECT_CONTENT &&
      request.lindex == stored.lindex && (request.tymed & stored.tymed) && !request.ptd;
  }
  LONG refs_ = 1;
  std::vector<Entry> entries_;
};
class Source final : public IDropSource {
 public:
  HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** result) override {
    if (!result) return E_POINTER;
    *result = nullptr;
    if (iid == IID_IUnknown || iid == IID_IDropSource) { *result = this; AddRef(); return S_OK; }
    return E_NOINTERFACE;
  }
  ULONG STDMETHODCALLTYPE AddRef() override { return InterlockedIncrement(&refs_); }
  ULONG STDMETHODCALLTYPE Release() override { ULONG n = InterlockedDecrement(&refs_); if (!n) delete this; return n; }
  HRESULT STDMETHODCALLTYPE QueryContinueDrag(BOOL escape, DWORD keys) override {
    if (escape) return DRAGDROP_S_CANCEL;
    if (!(keys & MK_LBUTTON)) return DRAGDROP_S_DROP;
    return S_OK;
  }
  HRESULT STDMETHODCALLTYPE GiveFeedback(DWORD) override { return DRAGDROP_S_USEDEFAULTCURSORS; }
 private:
  LONG refs_ = 1;
};
std::string UTF8(const std::wstring& value) {
  int count = WideCharToMultiByte(CP_UTF8, 0, value.data(), static_cast<int>(value.size()), nullptr, 0, nullptr, nullptr);
  std::string result(count, 0);
  WideCharToMultiByte(CP_UTF8, 0, value.data(), static_cast<int>(value.size()), result.data(), count, nullptr, nullptr);
  return result;
}
void AddHTML(Data* data, const std::wstring& html, const std::wstring& base) {
  std::string fragment = UTF8(html);
  const std::string before = "<html><body><!--StartFragment-->";
  const std::string after = "<!--EndFragment--></body></html>";
  std::string source = UTF8(base);
  source.erase(std::remove(source.begin(), source.end(), '\r'), source.end());
  source.erase(std::remove(source.begin(), source.end(), '\n'), source.end());
  auto header = [&](size_t start, size_t end, size_t first, size_t last) {
    char buffer[256];
    snprintf(buffer, sizeof(buffer), "Version:0.9\r\nStartHTML:%010zu\r\nEndHTML:%010zu\r\nStartFragment:%010zu\r\nEndFragment:%010zu\r\n", start, end, first, last);
    return std::string(buffer) + (source.empty() ? "" : "SourceURL:" + source + "\r\n");
  };
  size_t start = header(0, 0, 0, 0).size();
  std::string output = header(start, start + before.size() + fragment.size() + after.size(), start + before.size(), start + before.size() + fragment.size()) + before + fragment + after;
  output.push_back(0);
  data->Add(RegisterClipboardFormatW(L"HTML Format"), output.data(), output.size());
}
void AddFiles(Data* data, const RechromDragFile* files, size_t count) {
  if (!count) return;
  std::vector<wchar_t> paths;
  bool all_backed = true;
  for (size_t i = 0; i < count; ++i) {
    if (!files[i].path_len) { all_backed = false; break; }
    paths.insert(paths.end(), files[i].path, files[i].path + files[i].path_len); paths.push_back(0);
  }
  if (all_backed) {
    paths.push_back(0);
    std::vector<unsigned char> bytes(sizeof(DROPFILES) + paths.size() * sizeof(wchar_t), 0);
    auto* drop = reinterpret_cast<DROPFILES*>(bytes.data()); drop->pFiles = sizeof(DROPFILES); drop->fWide = TRUE;
    memcpy(bytes.data() + sizeof(DROPFILES), paths.data(), paths.size() * sizeof(wchar_t));
    data->Add(CF_HDROP, bytes.data(), bytes.size());
  }
  std::vector<unsigned char> descriptors(sizeof(UINT) + count * sizeof(FILEDESCRIPTORW), 0);
  auto* group = reinterpret_cast<FILEGROUPDESCRIPTORW*>(descriptors.data()); group->cItems = static_cast<UINT>(count);
  CLIPFORMAT contents = RegisterClipboardFormatW(CFSTR_FILECONTENTS);
  for (size_t i = 0; i < count; ++i) {
    auto& descriptor = group->fgd[i]; const auto& file = files[i];
    descriptor.dwFlags = FD_UNICODE | FD_ATTRIBUTES; descriptor.dwFileAttributes = FILE_ATTRIBUTE_NORMAL;
    // FILEDESCRIPTOR's native fixed field cannot carry a longer name.
    std::copy_n(file.name, std::min<size_t>(file.name_len, MAX_PATH - 1), descriptor.cFileName);
    if (file.path_len) {
      data->AddPath(contents, std::wstring(file.path, file.path_len), static_cast<LONG>(i));
    } else {
      descriptor.dwFlags |= FD_FILESIZE;
      descriptor.nFileSizeHigh = static_cast<DWORD>(static_cast<unsigned long long>(file.len) >> 32);
      descriptor.nFileSizeLow = static_cast<DWORD>(file.len);
      data->Add(contents, file.bytes, file.len, static_cast<LONG>(i), TYMED_ISTREAM);
    }
  }
  data->Add(RegisterClipboardFormatW(CFSTR_FILEDESCRIPTORW), descriptors.data(), descriptors.size());
}
void AddImage(Data* data, const RechromDragImage* image, int x, int y) {
  if (!image) return;
  IDragSourceHelper* helper = nullptr;
  if (FAILED(CoCreateInstance(CLSID_DragDropHelper, nullptr, CLSCTX_INPROC_SERVER, IID_PPV_ARGS(&helper)))) return;
  BITMAPINFO info = {}; info.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
  info.bmiHeader.biWidth = image->width; info.bmiHeader.biHeight = -static_cast<LONG>(image->height);
  info.bmiHeader.biPlanes = 1; info.bmiHeader.biBitCount = 32; info.bmiHeader.biCompression = BI_RGB;
  void* pixels = nullptr;
  HBITMAP bitmap = CreateDIBSection(nullptr, &info, DIB_RGB_COLORS, &pixels, nullptr, 0);
  if (bitmap) {
    auto* destination = static_cast<unsigned char*>(pixels);
    for (unsigned row = 0; row < image->height; ++row) for (unsigned column = 0; column < image->width; ++column) {
      const auto* source = image->rgba + row * image->stride + column * 4;
      auto* output = destination + (static_cast<size_t>(row) * image->width + column) * 4;
      // DragImageBitmap is premultiplied; the shell helper expects straight
      // color components, as in Chromium's UnPreMultiply before this call.
      auto straight = [&](unsigned char color) -> unsigned char {
        return source[3] ? static_cast<unsigned char>(std::min(255u, (unsigned(color) * 255u + source[3] / 2u) / source[3])) : 0;
      };
      output[0] = straight(source[2]); output[1] = straight(source[1]); output[2] = straight(source[0]); output[3] = source[3];
    }
    SHDRAGIMAGE drag = {{static_cast<LONG>(image->width), static_cast<LONG>(image->height)}, {x, y}, bitmap, CLR_NONE};
    // The helper takes bitmap ownership on success.
    if (FAILED(helper->InitializeFromBitmap(&drag, data))) DeleteObject(bitmap);
  }
  helper->Release();
}
}  // namespace

extern "C" int rechrom_do_drag_drop(void* window, const RechromDragString* strings, size_t strings_len,
    const RechromDragFile* files, size_t files_len, const RechromDragImage* image, int x, int y,
    unsigned operations, unsigned* result, int* point) {
  HRESULT initialized = OleInitialize(nullptr);
  if (FAILED(initialized)) return -1;
  auto* data = new Data;
  std::wstring base;
  for (size_t i = 0; i < strings_len; ++i) {
    std::wstring mime(strings[i].mime, strings[i].mime_len);
    if (mime == L"org.chromium.html-base-url") base.assign(strings[i].text, strings[i].len);
  }
  for (size_t i = 0; i < strings_len; ++i) {
    std::wstring mime(strings[i].mime, strings[i].mime_len), text(strings[i].text, strings[i].len);
    data->AddWide(RegisterClipboardFormatW(mime.c_str()), text);
    if (mime == L"text/plain") data->AddWide(CF_UNICODETEXT, text);
    else if (mime == L"text/html") AddHTML(data, text, base);
    else if (mime == L"text/uri-list") {
      size_t end = text.find_first_of(L"\r\n");
      data->AddWide(RegisterClipboardFormatW(CFSTR_INETURLW), text.substr(0, end));
    }
  }
  AddFiles(data, files, files_len); AddImage(data, image, x, y);
  auto* source = new Source;
  DWORD allowed = ((operations & 1) ? DROPEFFECT_COPY : 0) | ((operations & 2) ? DROPEFFECT_LINK : 0) | ((operations & 16) ? DROPEFFECT_MOVE : 0);
  DWORD effect = DROPEFFECT_NONE;
  HRESULT status = DoDragDrop(data, source, allowed, &effect);
  POINT cursor = {}; GetCursorPos(&cursor); point[2] = cursor.x; point[3] = cursor.y;
  ScreenToClient(static_cast<HWND>(window), &cursor); point[0] = cursor.x; point[1] = cursor.y;
  *result = effect == DROPEFFECT_MOVE ? 16 : effect == DROPEFFECT_LINK ? 2 : effect == DROPEFFECT_COPY ? 1 : 0;
  source->Release(); data->Release(); OleUninitialize();
  return status == DRAGDROP_S_DROP ? 1 : status == DRAGDROP_S_CANCEL ? 0 : -1;
}
