// SPDX-License-Identifier: GPL-3.0-or-later
// Read the actual macOS accessibility tree of a running showcase process.
import ApplicationServices
import Foundation

guard CommandLine.arguments.count == 3,
      let pid = Int32(CommandLine.arguments[1]) else {
    fatalError("usage: swift tools/audit-native-accessibility.swift PID output.json")
}
guard AXIsProcessTrusted() else {
    fatalError("Accessibility access is unavailable for this process")
}

func attribute(_ element: AXUIElement, _ name: CFString) -> CFTypeRef? {
    var value: CFTypeRef?
    guard AXUIElementCopyAttributeValue(element, name, &value) == .success else {
        return nil
    }
    return value
}

var count = 0
func inspect(_ element: AXUIElement, depth: Int) -> [String: Any] {
    count += 1
    var result: [String: Any] = [:]
    let attributes: [(String, CFString)] = [
        ("role", kAXRoleAttribute as CFString),
        ("roleDescription", kAXRoleDescriptionAttribute as CFString),
        ("title", kAXTitleAttribute as CFString),
        ("description", kAXDescriptionAttribute as CFString),
        ("value", kAXValueAttribute as CFString),
        ("enabled", kAXEnabledAttribute as CFString),
        ("focused", kAXFocusedAttribute as CFString),
        ("selected", kAXSelectedAttribute as CFString),
    ]
    for (key, name) in attributes {
        if let value = attribute(element, name) {
            if let text = value as? String {
                result[key] = text
            } else if let number = value as? NSNumber {
                result[key] = number
            }
        }
    }
    var actions: CFArray?
    if AXUIElementCopyActionNames(element, &actions) == .success,
       let names = actions as? [String] {
        result["actions"] = names
    }
    if depth < 24 && count < 3000,
       let children = attribute(element, kAXChildrenAttribute as CFString) as? [AXUIElement] {
        result["children"] = children.map { inspect($0, depth: depth + 1) }
    }
    return result
}

let tree = inspect(AXUIElementCreateApplication(pid), depth: 0)
let report: [String: Any] = [
    "pid": pid,
    "method": "macOS AXUIElement",
    "qualification": "Actual accessibility bridge tree only. Does not qualify a VoiceOver or IME flow.",
    "elementCount": count,
    "tree": tree,
]
let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
try data.write(to: URL(fileURLWithPath: CommandLine.arguments[2]))
print("Read \(count) native accessibility elements")
