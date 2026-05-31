import UIKit
import WidgetKit

/// Called from Rust after writing `widget/upcoming.json` to the App Group container.
@_cdecl("subly_reload_widget_timelines")
public func subly_reload_widget_timelines() {
    WidgetCenter.shared.reloadAllTimelines()
}

private var didInstallActivateObserver = false

/// Ensures the widget re-reads App Group JSON when the user opens the app (belt-and-suspenders with Rust export).
@_cdecl("subly_install_widget_reload_on_activate")
public func subly_install_widget_reload_on_activate() {
    guard !didInstallActivateObserver else { return }
    didInstallActivateObserver = true
    NotificationCenter.default.addObserver(
        forName: UIApplication.didBecomeActiveNotification,
        object: nil,
        queue: .main
    ) { _ in
        WidgetCenter.shared.reloadAllTimelines()
    }
}
