import WidgetKit

/// Called from Rust after writing `widget/upcoming.json` to the App Group container.
@_cdecl("subly_reload_widget_timelines")
public func subly_reload_widget_timelines() {
    WidgetCenter.shared.reloadAllTimelines()
}
