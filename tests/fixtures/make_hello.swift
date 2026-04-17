import AppKit

let text = "Hello, world."
let size = NSSize(width: 400, height: 120)
let image = NSImage(size: size)
image.lockFocus()
NSColor.white.setFill()
NSBezierPath(rect: NSRect(origin: .zero, size: size)).fill()
let attrs: [NSAttributedString.Key: Any] = [
    .font: NSFont.systemFont(ofSize: 48),
    .foregroundColor: NSColor.black,
]
let attr = NSAttributedString(string: text, attributes: attrs)
attr.draw(at: NSPoint(x: 20, y: 30))
image.unlockFocus()

guard let tiff = image.tiffRepresentation,
      let rep = NSBitmapImageRep(data: tiff),
      let png = rep.representation(using: .png, properties: [:]) else {
    FileHandle.standardError.write("failed to encode png\n".data(using: .utf8)!)
    exit(1)
}
let outPath = CommandLine.arguments.count > 1 ? CommandLine.arguments[1] : "hello.png"
try png.write(to: URL(fileURLWithPath: outPath))
