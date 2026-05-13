import UIKit
import Metal
import QuartzCore

final class MetalHostViewController: UIViewController {
    private var metalLayer: CAMetalLayer!

    override var isAccessibilityElement: Bool { false }

    override var accessibilityElements: [Any]? {
        get {
            let containerView = view
            var count: UInt = 0
            arcadia_ios_accessibility_node_count(&count)
            guard count > 0 else { return nil }

            let labelCap = 4096
            let hintCap = 4096
            let rectBuf = UnsafeMutablePointer<Float>.allocate(capacity: 4)
            let labelBuf = UnsafeMutablePointer<CChar>.allocate(capacity: labelCap)
            let hintBuf = UnsafeMutablePointer<CChar>.allocate(capacity: hintCap)
            defer {
                rectBuf.deallocate()
                labelBuf.deallocate()
                hintBuf.deallocate()
            }

            var elements: [UIAccessibilityElement] = []
            elements.reserveCapacity(Int(count))
            for idx in 0 ..< Int(count) {
                var traits: UInt64 = 0
                var labelLen: UInt = 0
                var hintLen: UInt = 0
                arcadia_ios_accessibility_query_node(
                    UInt(idx),
                    rectBuf,
                    &traits,
                    labelBuf,
                    UInt(labelCap),
                    hintBuf,
                    UInt(hintCap),
                    &labelLen,
                    &hintLen
                )
                let el = UIAccessibilityElement(accessibilityContainer: containerView)
                el.accessibilityFrameInContainerSpace = CGRect(
                    x: CGFloat(rectBuf[0]),
                    y: CGFloat(rectBuf[1]),
                    width: CGFloat(rectBuf[2]),
                    height: CGFloat(rectBuf[3])
                )
                el.accessibilityTraits = UIAccessibilityTraits(rawValue: traits)
                if labelLen > 0 {
                    el.accessibilityLabel = String(cString: labelBuf)
                }
                if hintLen > 0 {
                    el.accessibilityHint = String(cString: hintBuf)
                }
                elements.append(el)
            }
            return elements
        }
        set { }
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        metalLayer = CAMetalLayer()
        metalLayer.frame = view.bounds
        metalLayer.device = MTLCreateSystemDefaultDevice()
        metalLayer.pixelFormat = .bgra8Unorm
        metalLayer.framebufferOnly = true
        view.layer.addSublayer(metalLayer)

        let fm = FileManager.default
        let docs = fm.urls(for: .documentDirectory, in: .userDomainMask)[0].path

        let layerPtr = UInt(bitPattern: Unmanaged.passUnretained(metalLayer).toOpaque())
        arcadia_ios_start(layerPtr, docs)
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()
        metalLayer.frame = view.bounds
        UIAccessibility.post(notification: .layoutChanged, argument: nil)
    }

    override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        forwardTouches(touches, phase: 0)
    }
    override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        forwardTouches(touches, phase: 1)
    }
    override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        forwardTouches(touches, phase: 2)
    }
    override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        forwardTouches(touches, phase: 3)
    }

    private func forwardTouches(_ touches: Set<UITouch>, phase: UInt8) {
        guard let touch = touches.first else { return }
        let pt = touch.location(in: view)
        arcadia_ios_inject_touch(Float(pt.x), Float(pt.y), phase)
    }
}
