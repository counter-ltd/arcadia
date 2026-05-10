import UIKit
import Metal
import QuartzCore

final class MetalHostViewController: UIViewController {
    private var metalLayer: CAMetalLayer!

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
