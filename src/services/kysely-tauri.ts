import {
  SqliteAdapter,
  SqliteIntrospector,
  SqliteQueryCompiler,
  CompiledQuery,
  type Dialect,
  type Driver,
  type DatabaseConnection,
  type QueryResult,
  type TransactionSettings,
  type Kysely,
} from "kysely";
import Database from "@tauri-apps/plugin-sql";

let _rawDb: Database | null = null;

export function getRawDb(): Database {
  if (!_rawDb) throw new Error("Database not initialized");
  return _rawDb;
}

class TauriConnection implements DatabaseConnection {
  readonly #db: Database;

  constructor(db: Database) {
    this.#db = db;
  }

  async executeQuery<R>(compiledQuery: CompiledQuery): Promise<QueryResult<R>> {
    const { sql, parameters } = compiledQuery;
    const isRead = /^\s*(select|pragma|explain|with)/i.test(sql);

    if (isRead) {
      const rows = await this.#db.select<R[]>(sql, [...parameters]);
      return { rows };
    }

    const result = await this.#db.execute(sql, [...parameters]);
    return {
      insertId: result.lastInsertId != null ? BigInt(result.lastInsertId) : undefined,
      numAffectedRows: BigInt(result.rowsAffected),
      rows: [],
    };
  }

  async *streamQuery<R>(): AsyncIterableIterator<QueryResult<R>> {
    throw new Error("Streaming not supported in Tauri SQLite");
  }
}

class TauriSqliteDriver implements Driver {
  readonly #path: string;

  constructor(path: string) {
    this.#path = path;
  }

  async init(): Promise<void> {
    if (!_rawDb) {
      _rawDb = await Database.load(this.#path);
    }
  }

  async acquireConnection(): Promise<DatabaseConnection> {
    return new TauriConnection(_rawDb!);
  }

  async beginTransaction(conn: DatabaseConnection, _settings: TransactionSettings): Promise<void> {
    await conn.executeQuery(CompiledQuery.raw("BEGIN TRANSACTION"));
  }

  async commitTransaction(conn: DatabaseConnection): Promise<void> {
    await conn.executeQuery(CompiledQuery.raw("COMMIT"));
  }

  async rollbackTransaction(conn: DatabaseConnection): Promise<void> {
    await conn.executeQuery(CompiledQuery.raw("ROLLBACK"));
  }

  async releaseConnection(): Promise<void> {}

  async destroy(): Promise<void> {
    _rawDb = null;
  }
}

export class TauriSqliteDialect implements Dialect {
  readonly #path: string;

  constructor(config: { path: string }) {
    this.#path = config.path;
  }

  createDriver(): Driver {
    return new TauriSqliteDriver(this.#path);
  }

  createQueryCompiler() {
    return new SqliteQueryCompiler();
  }

  createAdapter() {
    return new SqliteAdapter();
  }

  createIntrospector(db: Kysely<any>) {
    return new SqliteIntrospector(db);
  }
}
