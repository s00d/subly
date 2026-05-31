import XCTest

final class WidgetSnapshotLoaderTests: XCTestCase {
    private var fixturesURL: URL {
        URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .appendingPathComponent("Fixtures", isDirectory: true)
    }

    private func fixtureURL(_ name: String) -> URL {
        fixturesURL.appendingPathComponent(name)
    }

    func testDecodeStringPriceMatchesRustContract() throws {
        let url = fixtureURL("upcoming_string_price.json")
        let result = WidgetSnapshotLoader.load(from: url)

        XCTAssertNil(result.errorKey)
        XCTAssertEqual(result.items.count, 1)
        let item = try XCTUnwrap(result.items.first)
        XCTAssertEqual(item.name, "Streaming")
        XCTAssertEqual(item.price, 12.5, accuracy: 0.001)
        XCTAssertEqual(item.currencyCode, "USD")
        XCTAssertEqual(item.iconFile, "icons/sub-widget-fixture-1.png")
    }

    func testDecodeLegacyNumberPrice() throws {
        let url = fixtureURL("upcoming_legacy_number_price.json")
        let result = WidgetSnapshotLoader.load(from: url)

        XCTAssertNil(result.errorKey)
        XCTAssertEqual(result.items.count, 1)
        XCTAssertEqual(try XCTUnwrap(result.items.first).price, 9.99, accuracy: 0.001)
    }

    func testMissingFileReturnsOpenApp() {
        let url = fixturesURL.appendingPathComponent("does_not_exist.json")
        let result = WidgetSnapshotLoader.load(from: url)
        XCTAssertEqual(result.errorKey, .openApp)
        XCTAssertTrue(result.items.isEmpty)
    }

    func testInvalidJsonReturnsBadData() {
        let data = Data("{ not valid json }".utf8)
        let result = WidgetSnapshotLoader.decode(data: data)
        XCTAssertEqual(result.errorKey, .badData)
    }

    func testStringPriceInRawJsonCannotUseDoubleOnlyDecoder() throws {
        let url = fixtureURL("upcoming_string_price.json")
        let data = try Data(contentsOf: url)
        struct StrictItem: Decodable {
            let price: Double
        }
        struct StrictFile: Decodable {
            let items: [StrictItem]
        }
        XCTAssertThrowsError(try JSONDecoder().decode(StrictFile.self, from: data))
    }

    func testLoadFromAppGroupUsesInjectedContainer() throws {
        let fm = FileManager()
        let base = fm.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try fm.createDirectory(at: base, withIntermediateDirectories: true)
        defer { try? fm.removeItem(at: base) }

        let widgetDir = base.appendingPathComponent("widget", isDirectory: true)
        try fm.createDirectory(at: widgetDir, withIntermediateDirectories: true)
        let dest = widgetDir.appendingPathComponent("upcoming.json")
        try Data(contentsOf: fixtureURL("upcoming_string_price.json")).write(to: dest)

        let result = WidgetSnapshotLoader.load(
            from: dest,
            fileManager: fm
        )
        XCTAssertNil(result.errorKey)
        XCTAssertEqual(result.items.first?.name, "Streaming")
    }
}
