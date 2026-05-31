import Foundation

/// App Group + relative path written by Rust (`widget_snapshot.rs`).
enum WidgetSnapshotPaths {
    static let appGroupId = "group.com.s00d.subly"
    static let jsonRelativePath = "widget/upcoming.json"
}

struct WidgetSnapshotItem: Decodable, Equatable {
    let id: String
    let name: String
    let nextPayment: String
    let price: Double
    let currencyId: String
    let currencyCode: String?
    let iconFile: String?

    init(
        id: String,
        name: String,
        nextPayment: String,
        price: Double,
        currencyId: String,
        currencyCode: String?,
        iconFile: String?
    ) {
        self.id = id
        self.name = name
        self.nextPayment = nextPayment
        self.price = price
        self.currencyId = currencyId
        self.currencyCode = currencyCode
        self.iconFile = iconFile
    }

    private enum CodingKeys: String, CodingKey {
        case id, name, nextPayment, price, currencyId, currencyCode, iconFile
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        id = try c.decode(String.self, forKey: .id)
        name = try c.decode(String.self, forKey: .name)
        nextPayment = try c.decode(String.self, forKey: .nextPayment)
        price = Self.decodePrice(from: c)
        currencyId = try c.decode(String.self, forKey: .currencyId)
        currencyCode = try c.decodeIfPresent(String.self, forKey: .currencyCode)
        iconFile = try c.decodeIfPresent(String.self, forKey: .iconFile)
    }

    /// Rust `money::serde_ipc` emits decimal strings; legacy snapshots may use JSON numbers.
    private static func decodePrice(from c: KeyedDecodingContainer<CodingKeys>) -> Double {
        if let n = try? c.decode(Double.self, forKey: .price) { return n }
        if let s = try? c.decode(String.self, forKey: .price) {
            let trimmed = s.trimmingCharacters(in: .whitespacesAndNewlines)
            if let n = Double(trimmed) { return n }
            if let dec = Decimal(string: trimmed, locale: Locale(identifier: "en_US_POSIX")) {
                return (dec as NSDecimalNumber).doubleValue
            }
        }
        return 0
    }
}

struct WidgetSnapshotFile: Decodable, Equatable {
    let updatedAt: String?
    let items: [WidgetSnapshotItem]
}

enum WidgetSnapshotErrorKey: String, Equatable {
    case noGroup = "widget_err_no_group"
    case openApp = "widget_err_open_app"
    case badData = "widget_err_bad_data"
}

struct WidgetSnapshotLoadResult: Equatable {
    let items: [WidgetSnapshotItem]
    let errorKey: WidgetSnapshotErrorKey?
}

enum WidgetSnapshotLoader {
    static func load(from url: URL, fileManager: FileManager = .default) -> WidgetSnapshotLoadResult {
        guard fileManager.fileExists(atPath: url.path) else {
            return WidgetSnapshotLoadResult(items: [], errorKey: .openApp)
        }
        guard let data = try? Data(contentsOf: url) else {
            return WidgetSnapshotLoadResult(items: [], errorKey: .openApp)
        }
        return decode(data: data)
    }

    static func loadFromAppGroup(
        appGroupId: String = WidgetSnapshotPaths.appGroupId,
        jsonRelativePath: String = WidgetSnapshotPaths.jsonRelativePath,
        fileManager: FileManager = .default
    ) -> WidgetSnapshotLoadResult {
        guard let base = fileManager.containerURL(forSecurityApplicationGroupIdentifier: appGroupId) else {
            return WidgetSnapshotLoadResult(items: [], errorKey: .noGroup)
        }
        let url = base.appendingPathComponent(jsonRelativePath)
        return load(from: url, fileManager: fileManager)
    }

    static func decode(data: Data) -> WidgetSnapshotLoadResult {
        let dec = JSONDecoder()
        guard let file = try? dec.decode(WidgetSnapshotFile.self, from: data) else {
            return WidgetSnapshotLoadResult(items: [], errorKey: .badData)
        }
        return WidgetSnapshotLoadResult(items: file.items, errorKey: nil)
    }
}
