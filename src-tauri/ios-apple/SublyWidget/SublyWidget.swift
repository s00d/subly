import SwiftUI
import UIKit
import WidgetKit

private let kAppGroup = "group.com.s00d.subly"
private let kJsonRelative = "widget/upcoming.json"

// MARK: - JSON

struct WidgetSnapshotItem: Codable {
    let id: String
    let name: String
    let nextPayment: String
    let price: Double
    let currencyId: String
    /// ISO 4217 from app catalog; optional for older snapshot files.
    let currencyCode: String?
    /// Relative path under the shared `widget/` folder (e.g. `icons/<id>.png`), written by the main app.
    let iconFile: String?
}

struct WidgetSnapshotFile: Codable {
    let updatedAt: String?
    let items: [WidgetSnapshotItem]
}

struct SimpleEntry: TimelineEntry {
    /// Moment this entry becomes active; date-dependent labels use this (not wall-clock) so midnight transitions work offline.
    let date: Date
    let items: [WidgetSnapshotItem]
    let errorKey: String?
}

private struct LoadedSnapshot {
    let items: [WidgetSnapshotItem]
    let errorKey: String?
}

// MARK: - Theme

private enum SublyWidgetTheme {
    static let accent = Color(red: 0.22, green: 0.47, blue: 0.98)
    static let accentDeep = Color(red: 0.12, green: 0.35, blue: 0.88)
    static let rowStroke = Color.primary.opacity(0.09)
    static let tileFill = Color.primary.opacity(0.045)
}

// MARK: - Currency (matches app: Intl + locale — NumberFormatter + ISO code)

private func isoCurrencyCode(for item: WidgetSnapshotItem) -> String {
    let raw = (item.currencyCode ?? "").trimmingCharacters(in: .whitespacesAndNewlines).uppercased()
    if raw.count == 3 { return raw }
    return "USD"
}

private func formatCurrency(amount: Double, isoCode: String) -> String {
    let f = NumberFormatter()
    f.numberStyle = .currency
    f.currencyCode = isoCode
    f.locale = .current
    f.maximumFractionDigits = 2
    let frac = abs(amount.truncatingRemainder(dividingBy: 1))
    f.minimumFractionDigits = frac < 0.001 ? 0 : 2
    return f.string(from: NSNumber(value: amount)) ?? String(format: "%.2f", amount)
}

private func priceLabel(_ item: WidgetSnapshotItem) -> String {
    formatCurrency(amount: item.price, isoCode: isoCurrencyCode(for: item))
}

/// Strip control chars / newlines so one widget row cannot expand vertically from bad JSON.
private func widgetSingleLineTitle(_ raw: String) -> String {
    raw
        .replacingOccurrences(of: "\n", with: " ")
        .replacingOccurrences(of: "\t", with: " ")
        .trimmingCharacters(in: .whitespacesAndNewlines)
}

/// Hard cap length so pathological snapshot data cannot blow SwiftUI layout memory.
private func widgetClippedTitle(_ raw: String, maxScalars: Int = 200) -> String {
    let s = widgetSingleLineTitle(raw)
    if s.count <= maxScalars { return s }
    let idx = s.index(s.startIndex, offsetBy: maxScalars, limitedBy: s.endIndex) ?? s.endIndex
    return String(s[..<idx]) + "…"
}

// MARK: - Provider

struct UpcomingProvider: TimelineProvider {
    /// Reload timeline to pick up new `upcoming.json` after user opens the app (snapshot refresh).
    private let jsonReloadInterval: TimeInterval = 6 * 3600

    func placeholder(in _: Context) -> SimpleEntry {
        SimpleEntry(
            date: Date(),
            items: [
                WidgetSnapshotItem(
                    id: "demo",
                    name: "Example Premium",
                    nextPayment: "2099-01-15",
                    price: 9.99,
                    currencyId: "",
                    currencyCode: "USD",
                    iconFile: nil
                ),
            ],
            errorKey: nil
        )
    }

    func getSnapshot(in _: Context, completion: @escaping (SimpleEntry) -> Void) {
        let snap = loadSnapshot()
        completion(SimpleEntry(date: Date(), items: snap.items, errorKey: snap.errorKey))
    }

    func getTimeline(in _: Context, completion: @escaping (Timeline<SimpleEntry>) -> Void) {
        let snap = loadSnapshot()
        let now = Date()
        let calendar = Calendar.current

        if snap.errorKey != nil {
            let entry = SimpleEntry(date: now, items: snap.items, errorKey: snap.errorKey)
            let retry = calendar.date(byAdding: .hour, value: 1, to: now) ?? now.addingTimeInterval(3600)
            completion(Timeline(entries: [entry], policy: .after(retry)))
            return
        }

        if snap.items.isEmpty {
            let entry = SimpleEntry(date: now, items: [], errorKey: nil)
            completion(Timeline(entries: [entry], policy: .after(now.addingTimeInterval(jsonReloadInterval))))
            return
        }

        var anchors: [Date] = [now]
        let startToday = calendar.startOfDay(for: now)
        for offset in 1 ... 36 {
            if let midnight = calendar.date(byAdding: .day, value: offset, to: startToday) {
                anchors.append(midnight)
            }
        }
        anchors.sort()
        let entries = anchors.map { SimpleEntry(date: $0, items: snap.items, errorKey: nil) }

        completion(Timeline(entries: entries, policy: .after(now.addingTimeInterval(jsonReloadInterval))))
    }

    private func loadSnapshot() -> LoadedSnapshot {
        guard let base = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: kAppGroup) else {
            return LoadedSnapshot(items: [], errorKey: "widget_err_no_group")
        }
        let url = base.appendingPathComponent(kJsonRelative)
        guard let data = try? Data(contentsOf: url) else {
            return LoadedSnapshot(items: [], errorKey: "widget_err_open_app")
        }
        let dec = JSONDecoder()
        guard let file = try? dec.decode(WidgetSnapshotFile.self, from: data) else {
            return LoadedSnapshot(items: [], errorKey: "widget_err_bad_data")
        }
        return LoadedSnapshot(items: file.items, errorKey: nil)
    }
}

// MARK: - Dates

private func parsePaymentDate(_ iso: String) -> Date? {
    let parts = iso.split(separator: "-").compactMap { Int($0) }
    guard parts.count == 3 else { return nil }
    var c = DateComponents()
    c.year = parts[0]
    c.month = parts[1]
    c.day = parts[2]
    return Calendar(identifier: .gregorian).date(from: c)
}

/// Labels relative to the timeline entry date so midnight transitions match stored JSON without opening the app.
private func paymentDateLabel(_ iso: String, referenceDate: Date) -> String {
    let trimmed = iso.trimmingCharacters(in: .whitespacesAndNewlines)
    guard let payDate = parsePaymentDate(trimmed) else { return formatDayShort(trimmed) }
    let cal = Calendar.current
    let refDay = cal.startOfDay(for: referenceDate)
    let payDay = cal.startOfDay(for: payDate)
    let days = cal.dateComponents([.day], from: refDay, to: payDay).day ?? 0
    if days == 0 {
        return String(localized: String.LocalizationValue("widget_date_today"))
    }
    if days == 1 {
        return String(localized: String.LocalizationValue("widget_date_tomorrow"))
    }
    return formatDayShort(trimmed)
}

private func formatDayShort(_ iso: String) -> String {
    let trimmed = iso.trimmingCharacters(in: .whitespacesAndNewlines)
    guard let date = parsePaymentDate(trimmed) else {
        return trimmed.count > 28 ? String(trimmed.prefix(28)) + "…" : trimmed
    }
    let f = DateFormatter()
    f.locale = .current
    f.dateStyle = .short
    f.timeStyle = .none
    return f.string(from: date)
}

// MARK: - Subscription avatar (raster from App Group, else first letter)

private func widgetRasterIconURL(iconFile: String?) -> URL? {
    guard let rel = iconFile?.trimmingCharacters(in: .whitespacesAndNewlines), !rel.isEmpty,
          !rel.contains(".."),
          let base = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: kAppGroup)
    else { return nil }
    let url = base.appendingPathComponent("widget").appendingPathComponent(rel)
    return FileManager.default.fileExists(atPath: url.path) ? url : nil
}

private struct SubscriptionAvatar: View {
    let name: String
    var iconFile: String? = nil
    var edge: CGFloat = 36

    private var glyph: String {
        let t = widgetSingleLineTitle(name)
        guard let ch = t.first else { return "?" }
        return String(ch).uppercased()
    }

    private var rasterURL: URL? {
        widgetRasterIconURL(iconFile: iconFile)
    }

    var body: some View {
        ZStack {
            if let url = rasterURL, let ui = UIImage(contentsOfFile: url.path) {
                Image(uiImage: ui)
                    .resizable()
                    .interpolation(.high)
                    .scaledToFill()
                    .frame(width: edge, height: edge)
                    .clipped()
            } else {
                RoundedRectangle(cornerRadius: edge * 0.26, style: .continuous)
                    .fill(
                        LinearGradient(
                            colors: [SublyWidgetTheme.accent, SublyWidgetTheme.accentDeep],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                Text(glyph)
                    .font(.system(size: edge * 0.38, weight: .bold, design: .rounded))
                    .foregroundStyle(.white)
            }
        }
        .clipShape(RoundedRectangle(cornerRadius: edge * 0.26, style: .continuous))
        .frame(width: edge, height: edge)
        .accessibilityHidden(true)
    }
}

// MARK: - Chrome & header

private struct WidgetChrome<Content: View>: View {
    @Environment(\.widgetFamily) private var family
    @ViewBuilder var content: () -> Content

    private var inset: CGFloat {
        switch family {
        case .systemMedium: 10
        case .systemLarge: 12
        default: 14
        }
    }

    @ViewBuilder
    var body: some View {
        if #available(iOSApplicationExtension 17.0, *) {
            content()
                .padding(inset)
                .containerBackground(for: .widget) {
                    ZStack {
                        Color(UIColor.secondarySystemBackground)
                        LinearGradient(
                            colors: [
                                SublyWidgetTheme.accent.opacity(0.11),
                                Color.clear,
                            ],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    }
                }
        } else {
            content()
                .padding(inset)
                .background(Color(UIColor.secondarySystemBackground))
        }
    }
}

// MARK: - Shared header & grid (medium + large — wide tiles use two columns)

private struct WidgetHeaderMedium: View {
    var body: some View {
        HStack(spacing: 5) {
            Image(systemName: "calendar.badge.clock")
                .font(.system(size: 11, weight: .semibold))
                .foregroundStyle(SublyWidgetTheme.accent)
                .symbolRenderingMode(.hierarchical)
            Text(String(localized: String.LocalizationValue("widget_upcoming_title")))
                .font(.system(size: 11, weight: .bold, design: .rounded))
                .foregroundStyle(.primary)
                .lineLimit(1)
                .minimumScaleFactor(0.72)
                .truncationMode(.tail)
            Spacer(minLength: 0)
        }
    }
}

private struct MediumCompactRow: View {
    let item: WidgetSnapshotItem
    let referenceDate: Date
    /// Tighter row for large-widget two-column grid (narrow cells).
    var dense: Bool = false

    private var avatarEdge: CGFloat { dense ? 20 : 24 }
    private var nameSize: CGFloat { dense ? 10 : 11 }
    private var metaSize: CGFloat { dense ? 8 : 9 }
    private var priceSize: CGFloat { dense ? 10 : 11 }

    var body: some View {
        HStack(alignment: .center, spacing: dense ? 5 : 6) {
            SubscriptionAvatar(name: item.name, iconFile: item.iconFile, edge: avatarEdge)
            VStack(alignment: .leading, spacing: dense ? 0 : 1) {
                Text(widgetClippedTitle(item.name))
                    .font(.system(size: nameSize, weight: .semibold, design: .rounded))
                    .foregroundStyle(.primary)
                    .lineLimit(1)
                    .minimumScaleFactor(dense ? 0.65 : 0.72)
                    .truncationMode(.tail)
                Text(paymentDateLabel(item.nextPayment, referenceDate: referenceDate))
                    .font(.system(size: metaSize, weight: .medium, design: .rounded))
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .minimumScaleFactor(0.62)
                    .truncationMode(.tail)
            }
            .frame(minWidth: 0, maxWidth: .infinity, alignment: .leading)
            .layoutPriority(0)

            Text(priceLabel(item))
                .font(.system(size: priceSize, weight: .bold, design: .rounded))
                .foregroundStyle(SublyWidgetTheme.accent)
                .monospacedDigit()
                .lineLimit(1)
                .minimumScaleFactor(0.52)
                .multilineTextAlignment(.trailing)
                .truncationMode(.tail)
                .layoutPriority(2)
                .fixedSize(horizontal: false, vertical: true)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.vertical, dense ? 3 : 4)
    }
}

// MARK: - Rows & states

private struct EmptyStateView: View {
    var body: some View {
        VStack(spacing: 10) {
            ZStack {
                Circle()
                    .fill(SublyWidgetTheme.accent.opacity(0.15))
                    .frame(width: 52, height: 52)
                Image(systemName: "checkmark.circle.fill")
                    .font(.system(size: 30))
                    .symbolRenderingMode(.hierarchical)
                    .foregroundStyle(SublyWidgetTheme.accent)
            }
            Text(String(localized: String.LocalizationValue("widget_no_upcoming")))
                .font(.system(size: 12, weight: .semibold, design: .rounded))
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .lineLimit(3)
                .minimumScaleFactor(0.78)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

private struct ErrorStateView: View {
    let key: String

    var body: some View {
        VStack(spacing: 10) {
            Image(systemName: "arrow.triangle.2.circlepath.icloud")
                .font(.system(size: 28))
                .foregroundStyle(SublyWidgetTheme.accent.opacity(0.8))
                .symbolRenderingMode(.hierarchical)
            Text(String(localized: String.LocalizationValue(key)))
                .font(.system(size: 11, weight: .semibold, design: .rounded))
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .lineLimit(4)
                .minimumScaleFactor(0.72)
                .padding(.horizontal, 4)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

private struct SmallWidgetBody: View {
    let item: WidgetSnapshotItem
    let referenceDate: Date

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Capsule()
                .fill(
                    LinearGradient(
                        colors: [SublyWidgetTheme.accent, SublyWidgetTheme.accent.opacity(0.55)],
                        startPoint: .leading,
                        endPoint: .trailing
                    )
                )
                .frame(height: 4)
                .padding(.bottom, 10)
            HStack(alignment: .top, spacing: 10) {
                SubscriptionAvatar(name: item.name, iconFile: item.iconFile, edge: 44)
                VStack(alignment: .leading, spacing: 6) {
                    Text(widgetClippedTitle(item.name))
                        .font(.system(size: 15, weight: .bold, design: .rounded))
                        .foregroundStyle(.primary)
                        .lineLimit(3)
                        .minimumScaleFactor(0.72)
                        .truncationMode(.tail)
                    HStack(spacing: 5) {
                        Image(systemName: "calendar.circle.fill")
                            .font(.system(size: 13))
                            .foregroundStyle(SublyWidgetTheme.accent.opacity(0.9))
                            .symbolRenderingMode(.hierarchical)
                            .frame(width: 16, alignment: .leading)
                        Text(paymentDateLabel(item.nextPayment, referenceDate: referenceDate))
                            .font(.system(size: 11, weight: .semibold, design: .rounded))
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                            .minimumScaleFactor(0.68)
                            .truncationMode(.tail)
                            .frame(minWidth: 0, maxWidth: .infinity, alignment: .leading)
                    }
                }
                .frame(minWidth: 0, maxWidth: .infinity, alignment: .leading)
            }
            Spacer(minLength: 8)
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Image(systemName: "banknote.fill")
                    .font(.system(size: 22))
                    .foregroundStyle(SublyWidgetTheme.accent)
                    .symbolRenderingMode(.hierarchical)
                Text(priceLabel(item))
                    .font(.system(size: 22, weight: .bold, design: .rounded))
                    .foregroundStyle(SublyWidgetTheme.accent)
                    .monospacedDigit()
                    .minimumScaleFactor(0.45)
                    .lineLimit(1)
                    .truncationMode(.tail)
                    .frame(minWidth: 0, maxWidth: .infinity, alignment: .leading)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
    }
}

/// Medium + Large home-screen tiles are wide on phone; both use two columns (not only `.systemLarge`).
private struct LargeWidgetBody: View {
    let items: [WidgetSnapshotItem]
    let referenceDate: Date

    private static let rowSlotHeight: CGFloat = 27
    private static let headerBlockHeight: CGFloat = 24
    private static let moreFooterHeight: CGFloat = 20
    private static let absoluteCellCap = 12

    private func visibleSlice(layoutHeight: CGFloat) -> (left: [WidgetSnapshotItem], right: [WidgetSnapshotItem], rest: Int) {
        guard !items.isEmpty else { return ([], [], 0) }
        let total = items.count

        func cellCountFitting(reserveMoreFooter: Bool) -> Int {
            var usable = layoutHeight - Self.headerBlockHeight
            if reserveMoreFooter {
                usable -= Self.moreFooterHeight
            }
            let rowsPerColumn = max(1, Int(floor(usable / Self.rowSlotHeight)))
            let cells = rowsPerColumn * 2
            return min(cells, Self.absoluteCellCap, total)
        }

        var n = cellCountFitting(reserveMoreFooter: false)
        if total > n {
            n = cellCountFitting(reserveMoreFooter: true)
        }

        let visible = Array(items.prefix(n))
        let rest = max(0, total - visible.count)
        let left = visible.enumerated().filter { $0.offset % 2 == 0 }.map(\.element)
        let right = visible.enumerated().filter { $0.offset % 2 == 1 }.map(\.element)
        return (left, right, rest)
    }

    @ViewBuilder
    private func columnStack(_ columnItems: [WidgetSnapshotItem]) -> some View {
        VStack(spacing: 0) {
            ForEach(Array(columnItems.enumerated()), id: \.element.id) { index, item in
                MediumCompactRow(item: item, referenceDate: referenceDate, dense: true)
                if index < columnItems.count - 1 {
                    Divider()
                        .background(Color.primary.opacity(0.12))
                        .padding(.leading, 26)
                }
            }
        }
        .frame(minWidth: 0, maxWidth: .infinity, alignment: .topLeading)
    }

    var body: some View {
        GeometryReader { geo in
            let slice = visibleSlice(layoutHeight: geo.size.height)
            let rest = slice.rest

            VStack(alignment: .leading, spacing: 0) {
                WidgetHeaderMedium()
                    .padding(.bottom, 3)
                    .layoutPriority(1)

                HStack(alignment: .top, spacing: 8) {
                    columnStack(slice.left)
                    columnStack(slice.right)
                }
                .layoutPriority(0)

                if rest > 0 {
                    HStack(spacing: 4) {
                        Image(systemName: "ellipsis.circle.fill")
                            .font(.system(size: 10))
                            .foregroundStyle(.tertiary)
                        Text(String(format: String(localized: String.LocalizationValue("widget_more_count")), rest))
                            .font(.system(size: 9, weight: .bold, design: .rounded))
                            .foregroundStyle(.tertiary)
                            .lineLimit(1)
                            .minimumScaleFactor(0.65)
                    }
                    .frame(maxWidth: .infinity, alignment: .center)
                    .padding(.top, 4)
                    .layoutPriority(1)
                }

                Spacer(minLength: 0)
            }
            .frame(width: geo.size.width, height: geo.size.height, alignment: .topLeading)
            .clipped()
        }
    }
}

struct UpcomingWidgetEntryView: View {
    @Environment(\.widgetFamily) private var family
    var entry: SimpleEntry

    var body: some View {
        Group {
            if let key = entry.errorKey {
                ErrorStateView(key: key)
            } else if entry.items.isEmpty {
                EmptyStateView()
            } else if family == .systemSmall {
                SmallWidgetBody(item: entry.items[0], referenceDate: entry.date)
            } else {
                LargeWidgetBody(items: entry.items, referenceDate: entry.date)
            }
        }
        .dynamicTypeSize(.medium ... .xxxLarge)
    }
}

struct UpcomingPaymentsWidget: Widget {
    let kind: String = "SublyUpcomingPaymentsWidget"

    var body: some WidgetConfiguration {
        StaticConfiguration(kind: kind, provider: UpcomingProvider()) { entry in
            WidgetChrome {
                UpcomingWidgetEntryView(entry: entry)
            }
        }
        .configurationDisplayName(Text(String(localized: String.LocalizationValue("widget_display_name"))))
        .description(Text(String(localized: String.LocalizationValue("widget_description"))))
        .supportedFamilies([.systemSmall, .systemMedium, .systemLarge])
    }
}

@main
struct SublyWidgetBundle: WidgetBundle {
    var body: some Widget {
        UpcomingPaymentsWidget()
    }
}
