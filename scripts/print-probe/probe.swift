// Baskı sondası (Plan 12 D2): arayüzün baskı önizlemesini, uygulamanın kullandığı yolla
// (WKWebView `printOperation(with:)`, kenar boşlukları 0; wry `print()` ile aynı) pencere
// göstermeden PDF'e yazdırır ve PDF'i denetler: sayfa sayısı, kâğıt boyutu, her sayfanın metni.
// Tauri IPC, arayüz fikstüründen (ui/test/fixtures/delhi.json) taklit edilir; gerçek danışan
// verisi okunmaz. Kullanım: probe <url> <fikstür.json> <çıktı.pdf> <a4|letter> <exact|unknown>
// `unknown`: doğum saati bilinmeyen harita (uyarı ve notlar basılır; en uzun sayfalar).
import AppKit
import PDFKit
import WebKit

func fail(_ message: String) -> Never {
  FileHandle.standardError.write("print-probe: \(message)\n".data(using: .utf8)!)
  exit(1)
}

let args = CommandLine.arguments
guard args.count == 6 else { fail("usage: probe <url> <fixture.json> <out.pdf> <a4|letter> <exact|unknown>") }
guard let url = URL(string: args[1]) else { fail("bad url") }
guard let fixture = try? String(contentsOfFile: args[2], encoding: .utf8) else { fail("fixture not readable") }
let out = URL(fileURLWithPath: args[3])
// Punto cinsinden kâğıt: A4 210 × 297 mm, Letter 8,5 × 11 in.
let paper = args[4] == "letter" ? NSSize(width: 612, height: 792) : NSSize(width: 595.28, height: 841.89)
let unknownTime = args[5] == "unknown"
let label = "\(args[4]) \(args[5])"
/// Her sayfada bulunması gereken metin (Türkçe arayüz, fikstür haritası).
let expected = ["Örnek Danışan", "D9 · Navamsa", "Vimshottari dasa", "Nakshatra"]

let mock = """
(() => {
  const f = \(fixture);
  if (\(unknownTime)) {
    f.opened.chart_record.birth.time = null;
    f.opened.chart_record.birth.time_certainty = "Unknown";
    f.opened.chart_record.resolved.warnings = ["TimeUnknownNoon"];
  }
  const answers = {
    restore_session: f.opened, get_app_prefs: f.prefs, transits_at: f.transit,
    list_clients: [f.opened.client], list_charts: [f.opened.chart_record],
    list_tags: [], list_notes: [], list_clients_by_tag: [], search_clients: [],
  };
  window.__TAURI_INTERNALS__ = {
    invoke: async (cmd) => structuredClone(cmd in answers ? answers[cmd] : null),
    transformCallback: () => 0,
    unregisterCallback: () => {},
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main", windowLabel: "main" } },
  };
})();
"""

final class Probe: NSObject, WKNavigationDelegate {
  let web: WKWebView
  let window: NSWindow
  var tries = 0

  override init() {
    let config = WKWebViewConfiguration()
    config.websiteDataStore = .nonPersistent()
    config.userContentController.addUserScript(WKUserScript(source: mock, injectionTime: .atDocumentStart, forMainFrameOnly: true))
    window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 1200, height: 800), styleMask: [.titled], backing: .buffered, defer: false)
    web = WKWebView(frame: window.contentView!.bounds, configuration: config)
    window.contentView!.addSubview(web)
    super.init()
    web.navigationDelegate = self
  }

  func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) { waitFor("document.querySelector('h1.topbar-client') !== null", then: openPreview) }
  func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) { fail("page did not load") }
  func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) { fail("page did not load (is the dev server running?)") }

  /// `condition` doğru olana dek 100 ms arayla dener; 10 saniyede olmazsa çıkar.
  func waitFor(_ condition: String, then: @escaping () -> Void) {
    web.evaluateJavaScript(condition) { value, _ in
      if value as? Bool == true { self.tries = 0; then(); return }
      self.tries += 1
      if self.tries > 100 { fail("timed out waiting for: \(condition)") }
      DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { self.waitFor(condition, then: then) }
    }
  }

  func openPreview() {
    web.evaluateJavaScript("window.dispatchEvent(new KeyboardEvent('keydown', { key: 'p', metaKey: true, cancelable: true })); true") { _, _ in
      self.waitFor("document.querySelectorAll('section.print-page').length === 4", then: self.print)
    }
  }

  func print() {
    let info = NSPrintInfo()
    info.paperSize = paper
    info.topMargin = 0
    info.bottomMargin = 0
    info.leftMargin = 0
    info.rightMargin = 0
    info.jobDisposition = .save
    info.dictionary()[NSPrintInfo.AttributeKey.jobSavingURL] = out
    let operation = web.printOperation(with: info)
    operation.showsPrintPanel = false
    operation.showsProgressPanel = false
    operation.view?.frame = web.bounds
    let title = operation.jobTitle ?? ""
    operation.runModal(for: window, delegate: nil, didRun: nil, contextInfo: nil)
    waitForFile(title: title)
  }

  func waitForFile(title: String) {
    guard let pdf = PDFDocument(url: out), pdf.pageCount > 0 else {
      tries += 1
      if tries > 100 { fail("no PDF was written") }
      DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { self.waitForFile(title: title) }
      return
    }
    // Yazım sürerken okunmuş olabilir: sayfa sayısı durulana dek bir kez daha bakılır.
    DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) { self.check(title: title) }
  }

  func check(title: String) {
    guard let pdf = PDFDocument(url: out) else { fail("PDF not readable") }
    guard pdf.pageCount == expected.count else { fail("\(label): expected \(expected.count) pages, got \(pdf.pageCount) (a sheet overflows its page)") }
    for (i, text) in expected.enumerated() {
      let page = pdf.page(at: i)!
      let box = page.bounds(for: .mediaBox)
      guard abs(box.width - paper.width) < 1, abs(box.height - paper.height) < 1 else { fail("page \(i + 1) is \(box.width) × \(box.height) pt") }
      guard (page.string ?? "").contains(text) else { fail("page \(i + 1) does not contain “\(text)” (text is not selectable or the page order changed)") }
    }
    guard !(pdf.string ?? "").contains("Yazdır…") else { fail("the toolbar was printed") }
    guard title == "Örnek Danışan – Doğum haritası" else { fail("job title is “\(title)”") }
    if unknownTime, !(pdf.page(at: 1)?.string ?? "").contains("Doğum saati bilinmiyor") { fail("the unknown-time note is missing on the charts page") }
    Swift.print("print-probe: \(label) ok, \(pdf.pageCount) pages, job title “\(title)”")
    exit(0)
  }
}

let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let probe = Probe()
probe.web.load(URLRequest(url: url))
app.run()
