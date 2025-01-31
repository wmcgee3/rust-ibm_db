//! # IBM_DB
//!
//! `ibm_db` is a library for connecting to DB2.

// suppress for the whole module with inner attribute...
#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    improper_ctypes
)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use std::error::Error;
use std::fmt;

extern crate encoding_rs;
pub extern crate odbc_safe;

pub use connection::Connection;
pub use diagnostics::{DiagnosticRecord, GetDiagRec};
pub use environment::*;
pub use ffi::*;
pub use result::Result;
pub use statement::*;

use odbc_object::OdbcObject;
pub use odbc_safe as safe;
use raii::Raii;
use result::{into_result, try_into_option, Return};

mod connection;
mod diagnostics;
mod environment;
mod ffi;
mod odbc_object;
mod raii;
mod result;
mod statement;

/// Reflects the ability of a type to expose a valid handle
pub trait Handle {
    type To;
    /// Returns a valid handle to the odbc type.
    unsafe fn handle(&self) -> *mut Self::To;
}
//Added for connection pooling

#[derive(Debug)]
pub struct ODBCConnectionManager {
    connection_string: String,
}

#[derive(Debug)]
pub struct ODBCConnectionManagerTx {
    connection_string: String,
}

pub struct ODBCConnection<'a, AC: safe::AutocommitMode>(Connection<'a, AC>);

unsafe impl Send for ODBCConnection<'static, safe::AutocommitOn> {}
unsafe impl Send for ODBCConnection<'static, safe::AutocommitOff> {}

impl<'a, AC: safe::AutocommitMode> ODBCConnection<'a, AC> {
    pub fn raw(&self) -> &Connection<'a, AC> {
        &self.0
    }
}

pub struct ODBCEnv(Environment<Version3>);

unsafe impl Sync for ODBCEnv {}

unsafe impl Send for ODBCEnv {}

#[derive(Debug)]
pub struct ODBCError(Box<dyn Error>);

lazy_static! {
    static ref ENV: ODBCEnv = ODBCEnv(create_environment_v3().unwrap());
}

impl Error for ODBCError {
    fn description(&self) -> &str {
        "Error connecting DB"
    }
}

impl fmt::Display for ODBCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Box<DiagnosticRecord>> for ODBCError {
    fn from(err: Box<DiagnosticRecord>) -> Self {
        println!("ODBC ERROR {}", err);
        ODBCError(Box::new(err))
    }
}

impl<E: 'static> From<std::sync::PoisonError<E>> for ODBCError {
    fn from(err: std::sync::PoisonError<E>) -> Self {
        ODBCError(Box::new(err))
    }
}

impl ODBCConnectionManager {
    /// Creates a new `ODBCConnectionManager`.
    pub fn new<S: Into<String>>(connection_string: S) -> ODBCConnectionManager {
        ODBCConnectionManager {
            connection_string: connection_string.into(),
        }
    }
}

impl ODBCConnectionManagerTx {
    /// Creates a new `ODBCConnectionManagerTx`.
    pub fn new<S: Into<String>>(connection_string: S) -> ODBCConnectionManagerTx {
        ODBCConnectionManagerTx {
            connection_string: connection_string.into(),
        }
    }
}
impl r2d2::ManageConnection for ODBCConnectionManager {
    type Connection = ODBCConnection<'static, safe::AutocommitOn>;
    type Error = ODBCError;

    fn connect(&self) -> std::result::Result<Self::Connection, Self::Error> {
        let env = &ENV.0;
        Ok(ODBCConnection(
            env.connect_with_connection_string(&self.connection_string)?,
        ))
    }

    fn is_valid(&self, _conn: &mut Self::Connection) -> std::result::Result<(), Self::Error> {
        //TODO
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        //TODO
        false
    }
}

impl r2d2::ManageConnection for ODBCConnectionManagerTx {
    type Connection = ODBCConnection<'static, safe::AutocommitOff>;
    type Error = ODBCError;

    fn connect(&self) -> std::result::Result<Self::Connection, Self::Error> {
        let env = &ENV.0;
        let conn = env.connect_with_connection_string(&self.connection_string)?;
        let conn_result = conn.disable_autocommit();
        match conn_result {
            Ok(conn) => Ok(ODBCConnection(conn)),
            _ => Err(ODBCError("Unable to use transactions".into())),
        }
    }

    fn is_valid(&self, _conn: &mut Self::Connection) -> std::result::Result<(), Self::Error> {
        //TODO
        Ok(())
    }

    fn has_broken(&self, _conn: &mut Self::Connection) -> bool {
        //TODO
        false
    }
}
//Ends
pub const DB2LINUX: u32 = 1;
pub const SQL_CMP_NA_ERRORS: u32 = 1;
pub const SQL_CMP_ROWS_AFFECTED: u32 = 2;
pub const SQL_CMP_STMTS_COMPLETED: u32 = 3;
pub const SQL_CMP_REF_INT_ROWS: u32 = 4;
pub const SQL_CONNECT_DB_APP2DB_CONVFACTOR: u32 = 0;
pub const SQL_CONNECT_DB_DB2APP_CONVFACTOR: u32 = 1;
pub const SQL_CONNECT_DB_UPDATEABILITY_IN_UOW: u32 = 2;
pub const SQL_CONNECT_DB_COMMIT_TYPE: u32 = 3;
pub const SQL_DB_UPDATEABLE: u32 = 1;
pub const SQL_DB_READ_ONLY: u32 = 2;
pub const SQL_DB_ONE_PHASE_COMMIT: u32 = 1;
pub const SQL_DB_ONE_PHASE_READ_ONLY: u32 = 2;
pub const SQL_DB_TWO_PHASE_COMMIT: u32 = 3;
pub const SQL_ERRD_NODE_NUM: u32 = 1;
pub const DB2CLI_VER: u32 = 784;
pub const _FEATURES_H: u32 = 1;
pub const _DEFAULT_SOURCE: u32 = 1;
pub const __USE_ISOC11: u32 = 1;
pub const __USE_ISOC99: u32 = 1;
pub const __USE_ISOC95: u32 = 1;
pub const __USE_POSIX_IMPLICITLY: u32 = 1;
pub const _POSIX_SOURCE: u32 = 1;
pub const _POSIX_C_SOURCE: u32 = 200809;
pub const __USE_POSIX: u32 = 1;
pub const __USE_POSIX2: u32 = 1;
pub const __USE_POSIX199309: u32 = 1;
pub const __USE_POSIX199506: u32 = 1;
pub const __USE_XOPEN2K: u32 = 1;
pub const __USE_XOPEN2K8: u32 = 1;
pub const _ATFILE_SOURCE: u32 = 1;
pub const __USE_MISC: u32 = 1;
pub const __USE_ATFILE: u32 = 1;
pub const __USE_FORTIFY_LEVEL: u32 = 0;
pub const __GLIBC_USE_DEPRECATED_GETS: u32 = 0;
pub const _STDC_PREDEF_H: u32 = 1;
pub const __STDC_IEC_559__: u32 = 1;
pub const __STDC_IEC_559_COMPLEX__: u32 = 1;
pub const __STDC_ISO_10646__: u32 = 201706;
pub const __STDC_NO_THREADS__: u32 = 1;
pub const __GNU_LIBRARY__: u32 = 6;
pub const __GLIBC__: u32 = 2;
pub const __GLIBC_MINOR__: u32 = 27;
pub const _SYS_CDEFS_H: u32 = 1;
pub const __glibc_c99_flexarr_available: u32 = 1;
pub const __WORDSIZE: u32 = 64;
pub const __WORDSIZE_TIME64_COMPAT32: u32 = 1;
pub const __SYSCALL_WORDSIZE: u32 = 64;
pub const __HAVE_GENERIC_SELECTION: u32 = 1;
pub const __GLIBC_USE_LIB_EXT2: u32 = 0;
pub const __GLIBC_USE_IEC_60559_BFP_EXT: u32 = 0;
pub const __GLIBC_USE_IEC_60559_FUNCS_EXT: u32 = 0;
pub const __GLIBC_USE_IEC_60559_TYPES_EXT: u32 = 0;
pub const _STDLIB_H: u32 = 1;
pub const WNOHANG: u32 = 1;
pub const WUNTRACED: u32 = 2;
pub const WSTOPPED: u32 = 2;
pub const WEXITED: u32 = 4;
pub const WCONTINUED: u32 = 8;
pub const WNOWAIT: u32 = 16777216;
pub const __WNOTHREAD: u32 = 536870912;
pub const __WALL: u32 = 1073741824;
pub const __WCLONE: u32 = 2147483648;
pub const __ENUM_IDTYPE_T: u32 = 1;
pub const __W_CONTINUED: u32 = 65535;
pub const __WCOREFLAG: u32 = 128;
pub const __HAVE_FLOAT128: u32 = 0;
pub const __HAVE_DISTINCT_FLOAT128: u32 = 0;
pub const __HAVE_FLOAT64X: u32 = 1;
pub const __HAVE_FLOAT64X_LONG_DOUBLE: u32 = 1;
pub const __HAVE_FLOAT16: u32 = 0;
pub const __HAVE_FLOAT32: u32 = 1;
pub const __HAVE_FLOAT64: u32 = 1;
pub const __HAVE_FLOAT32X: u32 = 1;
pub const __HAVE_FLOAT128X: u32 = 0;
pub const __HAVE_DISTINCT_FLOAT16: u32 = 0;
pub const __HAVE_DISTINCT_FLOAT32: u32 = 0;
pub const __HAVE_DISTINCT_FLOAT64: u32 = 0;
pub const __HAVE_DISTINCT_FLOAT32X: u32 = 0;
pub const __HAVE_DISTINCT_FLOAT64X: u32 = 0;
pub const __HAVE_DISTINCT_FLOAT128X: u32 = 0;
pub const __HAVE_FLOATN_NOT_TYPEDEF: u32 = 0;
pub const __ldiv_t_defined: u32 = 1;
pub const __lldiv_t_defined: u32 = 1;
pub const RAND_MAX: u32 = 2147483647;
pub const EXIT_FAILURE: u32 = 1;
pub const EXIT_SUCCESS: u32 = 0;
pub const _SYS_TYPES_H: u32 = 1;
pub const _BITS_TYPES_H: u32 = 1;
pub const _BITS_TYPESIZES_H: u32 = 1;
pub const __OFF_T_MATCHES_OFF64_T: u32 = 1;
pub const __INO_T_MATCHES_INO64_T: u32 = 1;
pub const __RLIM_T_MATCHES_RLIM64_T: u32 = 1;
pub const __FD_SETSIZE: u32 = 1024;
pub const __clock_t_defined: u32 = 1;
pub const __clockid_t_defined: u32 = 1;
pub const __time_t_defined: u32 = 1;
pub const __timer_t_defined: u32 = 1;
pub const _BITS_STDINT_INTN_H: u32 = 1;
pub const __BIT_TYPES_DEFINED__: u32 = 1;
pub const _ENDIAN_H: u32 = 1;
pub const __LITTLE_ENDIAN: u32 = 1234;
pub const __BIG_ENDIAN: u32 = 4321;
pub const __PDP_ENDIAN: u32 = 3412;
pub const __BYTE_ORDER: u32 = 1234;
pub const __FLOAT_WORD_ORDER: u32 = 1234;
pub const LITTLE_ENDIAN: u32 = 1234;
pub const BIG_ENDIAN: u32 = 4321;
pub const PDP_ENDIAN: u32 = 3412;
pub const BYTE_ORDER: u32 = 1234;
pub const _BITS_BYTESWAP_H: u32 = 1;
pub const _BITS_UINTN_IDENTITY_H: u32 = 1;
pub const _SYS_SELECT_H: u32 = 1;
pub const __FD_ZERO_STOS: &[u8; 6usize] = b"stosq\0";
pub const __sigset_t_defined: u32 = 1;
pub const __timeval_defined: u32 = 1;
pub const _STRUCT_TIMESPEC: u32 = 1;
pub const FD_SETSIZE: u32 = 1024;
pub const _SYS_SYSMACROS_H: u32 = 1;
pub const _BITS_SYSMACROS_H: u32 = 1;
pub const _BITS_PTHREADTYPES_COMMON_H: u32 = 1;
pub const _THREAD_SHARED_TYPES_H: u32 = 1;
pub const _BITS_PTHREADTYPES_ARCH_H: u32 = 1;
pub const __SIZEOF_PTHREAD_MUTEX_T: u32 = 40;
pub const __SIZEOF_PTHREAD_ATTR_T: u32 = 56;
pub const __SIZEOF_PTHREAD_RWLOCK_T: u32 = 56;
pub const __SIZEOF_PTHREAD_BARRIER_T: u32 = 32;
pub const __SIZEOF_PTHREAD_MUTEXATTR_T: u32 = 4;
pub const __SIZEOF_PTHREAD_COND_T: u32 = 48;
pub const __SIZEOF_PTHREAD_CONDATTR_T: u32 = 4;
pub const __SIZEOF_PTHREAD_RWLOCKATTR_T: u32 = 8;
pub const __SIZEOF_PTHREAD_BARRIERATTR_T: u32 = 4;
pub const __PTHREAD_MUTEX_LOCK_ELISION: u32 = 1;
pub const __PTHREAD_MUTEX_NUSERS_AFTER_KIND: u32 = 0;
pub const __PTHREAD_MUTEX_USE_UNION: u32 = 0;
pub const __PTHREAD_RWLOCK_INT_FLAGS_SHARED: u32 = 1;
pub const __PTHREAD_MUTEX_HAVE_PREV: u32 = 1;
pub const __have_pthread_attr_t: u32 = 1;
pub const _ALLOCA_H: u32 = 1;
pub const SQL_MAX_MESSAGE_LENGTH: u32 = 1024;
pub const SQL_MAX_ID_LENGTH: u32 = 128;
pub const SQL_DATE_LEN: u32 = 10;
pub const SQL_TIME_LEN: u32 = 8;
pub const SQL_TIMESTAMP_LEN: u32 = 19;
pub const SQL_TIMESTAMPTZ_LEN: u32 = 25;
pub const SQL_HANDLE_ENV: u32 = 1;
pub const SQL_HANDLE_DBC: u32 = 2;
pub const SQL_HANDLE_STMT: u32 = 3;
pub const SQL_HANDLE_DESC: u32 = 4;
pub const SQL_SUCCESS: u32 = 0;
pub const SQL_SUCCESS_WITH_INFO: u32 = 1;
pub const SQL_NEED_DATA: u32 = 99;
pub const SQL_NO_DATA: u32 = 100;
pub const SQL_STILL_EXECUTING: u32 = 2;
pub const SQL_ERROR: i32 = -1;
pub const SQL_INVALID_HANDLE: i32 = -2;
pub const SQL_CLOSE: u32 = 0;
pub const SQL_DROP: u32 = 1;
pub const SQL_UNBIND: u32 = 2;
pub const SQL_RESET_PARAMS: u32 = 3;
pub const SQL_COMMIT: u32 = 0;
pub const SQL_ROLLBACK: u32 = 1;
pub const SQL_UNKNOWN_TYPE: u32 = 0;
pub const SQL_CHAR: u32 = 1;
pub const SQL_NUMERIC: u32 = 2;
pub const SQL_DECIMAL: u32 = 3;
pub const SQL_INTEGER: u32 = 4;
pub const SQL_SMALLINT: u32 = 5;
pub const SQL_FLOAT: u32 = 6;
pub const SQL_REAL: u32 = 7;
pub const SQL_DOUBLE: u32 = 8;
pub const SQL_DATETIME: u32 = 9;
pub const SQL_VARCHAR: u32 = 12;
pub const SQL_BOOLEAN: u32 = 16;
pub const SQL_ROW: u32 = 19;
pub const SQL_WCHAR: i32 = -8;
pub const SQL_WVARCHAR: i32 = -9;
pub const SQL_WLONGVARCHAR: i32 = -10;
pub const SQL_DECFLOAT: i32 = -360;
pub const SQL_TYPE_DATE: u32 = 91;
pub const SQL_TYPE_TIME: u32 = 92;
pub const SQL_TYPE_TIMESTAMP: u32 = 93;
pub const SQL_TYPE_TIMESTAMP_WITH_TIMEZONE: u32 = 95;
pub const SQL_UNSPECIFIED: u32 = 0;
pub const SQL_INSENSITIVE: u32 = 1;
pub const SQL_SENSITIVE: u32 = 2;
pub const SQL_DEFAULT: u32 = 99;
pub const SQL_ARD_TYPE: i32 = -99;
pub const SQL_CODE_DATE: u32 = 1;
pub const SQL_CODE_TIME: u32 = 2;
pub const SQL_CODE_TIMESTAMP: u32 = 3;
pub const SQL_CODE_TIMESTAMP_WITH_TIMEZONE: u32 = 4;
pub const SQL_GRAPHIC: i32 = -95;
pub const SQL_VARGRAPHIC: i32 = -96;
pub const SQL_LONGVARGRAPHIC: i32 = -97;
pub const SQL_BLOB: i32 = -98;
pub const SQL_CLOB: i32 = -99;
pub const SQL_DBCLOB: i32 = -350;
pub const SQL_XML: i32 = -370;
pub const SQL_CURSORHANDLE: i32 = -380;
pub const SQL_DATALINK: i32 = -400;
pub const SQL_USER_DEFINED_TYPE: i32 = -450;
pub const SQL_C_DBCHAR: i32 = -350;
pub const SQL_C_DECIMAL_IBM: u32 = 3;
pub const SQL_C_PTR: u32 = 2463;
pub const SQL_C_DECIMAL_OLEDB: u32 = 2514;
pub const SQL_C_DECIMAL64: i32 = -360;
pub const SQL_C_DECIMAL128: i32 = -361;
pub const SQL_C_TIMESTAMP_EXT: i32 = -362;
pub const SQL_C_TYPE_TIMESTAMP_EXT: i32 = -362;
pub const SQL_C_BINARYXML: i32 = -363;
pub const SQL_C_TIMESTAMP_EXT_TZ: i32 = -364;
pub const SQL_C_TYPE_TIMESTAMP_EXT_TZ: i32 = -364;
pub const SQL_C_CURSORHANDLE: i32 = -365;
pub const SQL_BLOB_LOCATOR: u32 = 31;
pub const SQL_CLOB_LOCATOR: u32 = 41;
pub const SQL_DBCLOB_LOCATOR: i32 = -351;
pub const SQL_C_BLOB_LOCATOR: u32 = 31;
pub const SQL_C_CLOB_LOCATOR: u32 = 41;
pub const SQL_C_DBCLOB_LOCATOR: i32 = -351;
pub const SQL_NO_NULLS: u32 = 0;
pub const SQL_NULLABLE: u32 = 1;
pub const SQL_NULLABLE_UNKNOWN: u32 = 2;
pub const SQL_NAMED: u32 = 0;
pub const SQL_UNNAMED: u32 = 1;
pub const SQL_DESC_ALLOC_AUTO: u32 = 1;
pub const SQL_DESC_ALLOC_USER: u32 = 2;
pub const SQL_TYPE_BASE: u32 = 0;
pub const SQL_TYPE_DISTINCT: u32 = 1;
pub const SQL_TYPE_STRUCTURED: u32 = 2;
pub const SQL_TYPE_REFERENCE: u32 = 3;
pub const SQL_NULL_DATA: i32 = -1;
pub const SQL_DATA_AT_EXEC: i32 = -2;
pub const SQL_NTS: i32 = -3;
pub const SQL_NTSL: i32 = -3;
pub const SQL_COLUMN_SCHEMA_NAME: u32 = 16;
pub const SQL_COLUMN_CATALOG_NAME: u32 = 17;
pub const SQL_COLUMN_DISTINCT_TYPE: u32 = 1250;
pub const SQL_DESC_DISTINCT_TYPE: u32 = 1250;
pub const SQL_COLUMN_REFERENCE_TYPE: u32 = 1251;
pub const SQL_DESC_REFERENCE_TYPE: u32 = 1251;
pub const SQL_DESC_STRUCTURED_TYPE: u32 = 1252;
pub const SQL_DESC_USER_TYPE: u32 = 1253;
pub const SQL_DESC_BASE_TYPE: u32 = 1254;
pub const SQL_DESC_KEY_TYPE: u32 = 1255;
pub const SQL_DESC_KEY_MEMBER: u32 = 1266;
pub const SQL_DESC_IDENTITY_VALUE: u32 = 1267;
pub const SQL_DESC_CODEPAGE: u32 = 1268;
pub const SQL_DESC_COUNT: u32 = 1001;
pub const SQL_DESC_TYPE: u32 = 1002;
pub const SQL_DESC_LENGTH: u32 = 1003;
pub const SQL_DESC_OCTET_LENGTH_PTR: u32 = 1004;
pub const SQL_DESC_PRECISION: u32 = 1005;
pub const SQL_DESC_SCALE: u32 = 1006;
pub const SQL_DESC_DATETIME_INTERVAL_CODE: u32 = 1007;
pub const SQL_DESC_NULLABLE: u32 = 1008;
pub const SQL_DESC_INDICATOR_PTR: u32 = 1009;
pub const SQL_DESC_DATA_PTR: u32 = 1010;
pub const SQL_DESC_NAME: u32 = 1011;
pub const SQL_DESC_UNNAMED: u32 = 1012;
pub const SQL_DESC_OCTET_LENGTH: u32 = 1013;
pub const SQL_DESC_ALLOC_TYPE: u32 = 1099;
pub const SQL_DESC_USER_DEFINED_TYPE_CODE: u32 = 1098;
pub const SQL_DESC_CARDINALITY: u32 = 1040;
pub const SQL_DESC_CARDINALITY_PTR: u32 = 1043;
pub const SQL_DESC_ROW_DESC: u32 = 1044;
pub const SQL_KEYTYPE_NONE: u32 = 0;
pub const SQL_KEYTYPE_PRIMARYKEY: u32 = 1;
pub const SQL_KEYTYPE_UNIQUEINDEX: u32 = 2;
pub const SQL_UPDT_READONLY: u32 = 0;
pub const SQL_UPDT_WRITE: u32 = 1;
pub const SQL_UPDT_READWRITE_UNKNOWN: u32 = 2;
pub const SQL_PRED_NONE: u32 = 0;
pub const SQL_PRED_CHAR: u32 = 1;
pub const SQL_PRED_BASIC: u32 = 2;
pub const SQL_NULL_HENV: u32 = 0;
pub const SQL_NULL_HDBC: u32 = 0;
pub const SQL_NULL_HSTMT: u32 = 0;
pub const SQL_NULL_HDESC: u32 = 0;
pub const SQL_NULL_HANDLE: u32 = 0;
pub const SQL_DIAG_RETURNCODE: u32 = 1;
pub const SQL_DIAG_NUMBER: u32 = 2;
pub const SQL_DIAG_ROW_COUNT: u32 = 3;
pub const SQL_DIAG_SQLSTATE: u32 = 4;
pub const SQL_DIAG_NATIVE: u32 = 5;
pub const SQL_DIAG_MESSAGE_TEXT: u32 = 6;
pub const SQL_DIAG_DYNAMIC_FUNCTION: u32 = 7;
pub const SQL_DIAG_CLASS_ORIGIN: u32 = 8;
pub const SQL_DIAG_SUBCLASS_ORIGIN: u32 = 9;
pub const SQL_DIAG_CONNECTION_NAME: u32 = 10;
pub const SQL_DIAG_SERVER_NAME: u32 = 11;
pub const SQL_DIAG_DYNAMIC_FUNCTION_CODE: u32 = 12;
pub const SQL_DIAG_ISAM_ERROR: u32 = 13;
pub const SQL_DIAG_SYSPLEX_STATISTICS: u32 = 2528;
pub const SQL_DIAG_DB2ZLOAD_RETCODE: u32 = 2529;
pub const SQL_DIAG_DB2ZLOAD_LOAD_MSGS: u32 = 2530;
pub const SQL_DIAG_LOG_FILENAME: u32 = 2531;
pub const SQL_DIAG_BAD_FILENAME: u32 = 2532;
pub const SQL_DIAG_ALTER_TABLE: u32 = 4;
pub const SQL_DIAG_CALL: u32 = 7;
pub const SQL_DIAG_CREATE_INDEX: i32 = -1;
pub const SQL_DIAG_CREATE_TABLE: u32 = 77;
pub const SQL_DIAG_CREATE_VIEW: u32 = 84;
pub const SQL_DIAG_DELETE_WHERE: u32 = 19;
pub const SQL_DIAG_DROP_INDEX: i32 = -2;
pub const SQL_DIAG_DROP_TABLE: u32 = 32;
pub const SQL_DIAG_DROP_VIEW: u32 = 36;
pub const SQL_DIAG_DYNAMIC_DELETE_CURSOR: u32 = 38;
pub const SQL_DIAG_DYNAMIC_UPDATE_CURSOR: u32 = 81;
pub const SQL_DIAG_GRANT: u32 = 48;
pub const SQL_DIAG_INSERT: u32 = 50;
pub const SQL_DIAG_MERGE: u32 = 128;
pub const SQL_DIAG_REVOKE: u32 = 59;
pub const SQL_DIAG_SELECT_CURSOR: u32 = 85;
pub const SQL_DIAG_UNKNOWN_STATEMENT: u32 = 0;
pub const SQL_DIAG_UPDATE_WHERE: u32 = 82;
pub const SQL_DIAG_DEFERRED_PREPARE_ERROR: u32 = 1279;
pub const SQL_ROW_NO_ROW_NUMBER: i32 = -1;
pub const SQL_ROW_NUMBER_UNKNOWN: i32 = -2;
pub const SQL_COLUMN_NO_COLUMN_NUMBER: i32 = -1;
pub const SQL_COLUMN_NUMBER_UNKNOWN: i32 = -2;
pub const SQL_MAX_C_NUMERIC_PRECISION: u32 = 38;
pub const SQL_MAX_NUMERIC_LEN: u32 = 16;
pub const SQL_DECIMAL64_LEN: u32 = 8;
pub const SQL_DECIMAL128_LEN: u32 = 16;
pub const ODBCVER: u32 = 896;
pub const SQL_SPEC_MAJOR: u32 = 3;
pub const SQL_SPEC_MINOR: u32 = 80;
pub const SQL_SPEC_STRING: &[u8; 6usize] = b"03.80\0";
pub const SQL_SQLSTATE_SIZE: u32 = 5;
pub const SQL_MAX_DSN_LENGTH: u32 = 32;
pub const SQL_MAX_OPTION_STRING_LENGTH: u32 = 256;
pub const SQL_NO_DATA_FOUND: u32 = 100;
pub const SQL_HANDLE_SENV: u32 = 5;
pub const SQL_ATTR_ODBC_VERSION: u32 = 200;
pub const SQL_ATTR_CONNECTION_POOLING: u32 = 201;
pub const SQL_ATTR_CP_MATCH: u32 = 202;
pub const SQL_CP_OFF: u32 = 0;
pub const SQL_CP_ONE_PER_DRIVER: u32 = 1;
pub const SQL_CP_ONE_PER_HENV: u32 = 2;
pub const SQL_CP_DEFAULT: u32 = 0;
pub const SQL_CP_STRICT_MATCH: u32 = 0;
pub const SQL_CP_RELAXED_MATCH: u32 = 1;
pub const SQL_CP_MATCH_DEFAULT: u32 = 0;
pub const SQL_OV_ODBC2: u32 = 2;
pub const SQL_OV_ODBC3: u32 = 3;
pub const SQL_OV_ODBC3_80: u32 = 380;
pub const SQL_ACCESS_MODE: u32 = 101;
pub const SQL_AUTOCOMMIT: u32 = 102;
pub const SQL_LOGIN_TIMEOUT: u32 = 103;
pub const SQL_OPT_TRACE: u32 = 104;
pub const SQL_OPT_TRACEFILE: u32 = 105;
pub const SQL_TRANSLATE_DLL: u32 = 106;
pub const SQL_TRANSLATE_OPTION: u32 = 107;
pub const SQL_TXN_ISOLATION: u32 = 108;
pub const SQL_CURRENT_QUALIFIER: u32 = 109;
pub const SQL_ODBC_CURSORS: u32 = 110;
pub const SQL_QUIET_MODE: u32 = 111;
pub const SQL_PACKET_SIZE: u32 = 112;
pub const SQL_ATTR_ACCESS_MODE: u32 = 101;
pub const SQL_ATTR_AUTOCOMMIT: u32 = 102;
pub const SQL_ATTR_CONNECTION_TIMEOUT: u32 = 113;
pub const SQL_ATTR_CURRENT_CATALOG: u32 = 109;
pub const SQL_ATTR_DISCONNECT_BEHAVIOR: u32 = 114;
pub const SQL_ATTR_ENLIST_IN_DTC: u32 = 1207;
pub const SQL_ATTR_ENLIST_IN_XA: u32 = 1208;
pub const SQL_ATTR_LOGIN_TIMEOUT: u32 = 103;
pub const SQL_ATTR_ODBC_CURSORS: u32 = 110;
pub const SQL_ATTR_PACKET_SIZE: u32 = 112;
pub const SQL_ATTR_QUIET_MODE: u32 = 111;
pub const SQL_ATTR_TRACE: u32 = 104;
pub const SQL_ATTR_TRACEFILE: u32 = 105;
pub const SQL_ATTR_TRANSLATE_LIB: u32 = 106;
pub const SQL_ATTR_TRANSLATE_OPTION: u32 = 107;
pub const SQL_ATTR_TXN_ISOLATION: u32 = 108;
pub const SQL_ATTR_CONNECTION_DEAD: u32 = 1209;
pub const SQL_ATTR_ANSI_APP: u32 = 115;
pub const SQL_ATTR_RESET_CONNECTION: u32 = 116;
pub const SQL_ATTR_ASYNC_DBC_FUNCTIONS_ENABLE: u32 = 117;
pub const SQL_MODE_READ_WRITE: u32 = 0;
pub const SQL_MODE_READ_ONLY: u32 = 1;
pub const SQL_MODE_DEFAULT: u32 = 0;
pub const SQL_AUTOCOMMIT_OFF: u32 = 0;
pub const SQL_AUTOCOMMIT_ON: u32 = 1;
pub const SQL_AUTOCOMMIT_DEFERRED: u32 = 2;
pub const SQL_AUTOCOMMIT_DEFAULT: u32 = 1;
pub const SQL_LOGIN_TIMEOUT_DEFAULT: u32 = 15;
pub const SQL_OPT_TRACE_OFF: u32 = 0;
pub const SQL_OPT_TRACE_ON: u32 = 1;
pub const SQL_OPT_TRACE_DEFAULT: u32 = 0;
pub const SQL_OPT_TRACE_FILE_DEFAULT: &[u8; 9usize] = b"\\SQL.LOG\0";
pub const SQL_CUR_USE_IF_NEEDED: u32 = 0;
pub const SQL_CUR_USE_ODBC: u32 = 1;
pub const SQL_CUR_USE_DRIVER: u32 = 2;
pub const SQL_CUR_DEFAULT: u32 = 2;
pub const SQL_DB_RETURN_TO_POOL: u32 = 0;
pub const SQL_DB_DISCONNECT: u32 = 1;
pub const SQL_DB_DEFAULT: u32 = 0;
pub const SQL_DTC_DONE: u32 = 0;
pub const SQL_CD_TRUE: u32 = 1;
pub const SQL_CD_FALSE: u32 = 0;
pub const SQL_AA_TRUE: u32 = 1;
pub const SQL_AA_FALSE: u32 = 0;
pub const SQL_RESET_CONNECTION_YES: u32 = 1;
pub const SQL_ASYNC_DBC_ENABLE_ON: u32 = 1;
pub const SQL_ASYNC_DBC_ENABLE_OFF: u32 = 0;
pub const SQL_ASYNC_DBC_ENABLE_DEFAULT: u32 = 0;
pub const SQL_QUERY_TIMEOUT: u32 = 0;
pub const SQL_MAX_ROWS: u32 = 1;
pub const SQL_NOSCAN: u32 = 2;
pub const SQL_MAX_LENGTH: u32 = 3;
pub const SQL_ASYNC_ENABLE: u32 = 4;
pub const SQL_BIND_TYPE: u32 = 5;
pub const SQL_CURSOR_TYPE: u32 = 6;
pub const SQL_CONCURRENCY: u32 = 7;
pub const SQL_KEYSET_SIZE: u32 = 8;
pub const SQL_ROWSET_SIZE: u32 = 9;
pub const SQL_SIMULATE_CURSOR: u32 = 10;
pub const SQL_RETRIEVE_DATA: u32 = 11;
pub const SQL_USE_BOOKMARKS: u32 = 12;
pub const SQL_GET_BOOKMARK: u32 = 13;
pub const SQL_ROW_NUMBER: u32 = 14;
pub const SQL_ATTR_ASYNC_ENABLE: u32 = 4;
pub const SQL_ATTR_CONCURRENCY: u32 = 7;
pub const SQL_ATTR_CURSOR_TYPE: u32 = 6;
pub const SQL_ATTR_ENABLE_AUTO_IPD: u32 = 15;
pub const SQL_ATTR_FETCH_BOOKMARK_PTR: u32 = 16;
pub const SQL_ATTR_KEYSET_SIZE: u32 = 8;
pub const SQL_ATTR_MAX_LENGTH: u32 = 3;
pub const SQL_ATTR_MAX_ROWS: u32 = 1;
pub const SQL_ATTR_NOSCAN: u32 = 2;
pub const SQL_ATTR_PARAM_BIND_OFFSET_PTR: u32 = 17;
pub const SQL_ATTR_PARAM_BIND_TYPE: u32 = 18;
pub const SQL_ATTR_PARAM_OPERATION_PTR: u32 = 19;
pub const SQL_ATTR_PARAM_STATUS_PTR: u32 = 20;
pub const SQL_ATTR_PARAMS_PROCESSED_PTR: u32 = 21;
pub const SQL_ATTR_PARAMSET_SIZE: u32 = 22;
pub const SQL_ATTR_QUERY_TIMEOUT: u32 = 0;
pub const SQL_ATTR_RETRIEVE_DATA: u32 = 11;
pub const SQL_ATTR_ROW_BIND_OFFSET_PTR: u32 = 23;
pub const SQL_ATTR_ROW_BIND_TYPE: u32 = 5;
pub const SQL_ATTR_ROW_NUMBER: u32 = 14;
pub const SQL_ATTR_ROW_OPERATION_PTR: u32 = 24;
pub const SQL_ATTR_ROW_STATUS_PTR: u32 = 25;
pub const SQL_ATTR_ROWS_FETCHED_PTR: u32 = 26;
pub const SQL_ATTR_ROW_ARRAY_SIZE: u32 = 27;
pub const SQL_ATTR_SIMULATE_CURSOR: u32 = 10;
pub const SQL_ATTR_USE_BOOKMARKS: u32 = 12;
pub const SQL_IS_POINTER: i32 = -4;
pub const SQL_IS_UINTEGER: i32 = -5;
pub const SQL_IS_INTEGER: i32 = -6;
pub const SQL_IS_USMALLINT: i32 = -7;
pub const SQL_IS_SMALLINT: i32 = -8;
pub const SQL_PARAM_BIND_BY_COLUMN: u32 = 0;
pub const SQL_PARAM_BIND_TYPE_DEFAULT: u32 = 0;
pub const SQL_QUERY_TIMEOUT_DEFAULT: u32 = 0;
pub const SQL_MAX_ROWS_DEFAULT: u32 = 0;
pub const SQL_NOSCAN_OFF: u32 = 0;
pub const SQL_NOSCAN_ON: u32 = 1;
pub const SQL_NOSCAN_DEFAULT: u32 = 0;
pub const SQL_MAX_LENGTH_DEFAULT: u32 = 0;
pub const SQL_ASYNC_ENABLE_OFF: u32 = 0;
pub const SQL_ASYNC_ENABLE_ON: u32 = 1;
pub const SQL_ASYNC_ENABLE_DEFAULT: u32 = 0;
pub const SQL_BIND_BY_COLUMN: u32 = 0;
pub const SQL_BIND_TYPE_DEFAULT: u32 = 0;
pub const SQL_CONCUR_READ_ONLY: u32 = 1;
pub const SQL_CONCUR_LOCK: u32 = 2;
pub const SQL_CONCUR_ROWVER: u32 = 3;
pub const SQL_CONCUR_VALUES: u32 = 4;
pub const SQL_CONCUR_DEFAULT: u32 = 1;
pub const SQL_CURSOR_FORWARD_ONLY: u32 = 0;
pub const SQL_CURSOR_KEYSET_DRIVEN: u32 = 1;
pub const SQL_CURSOR_DYNAMIC: u32 = 2;
pub const SQL_CURSOR_STATIC: u32 = 3;
pub const SQL_CURSOR_TYPE_DEFAULT: u32 = 0;
pub const SQL_ROWSET_SIZE_DEFAULT: u32 = 1;
pub const SQL_KEYSET_SIZE_DEFAULT: u32 = 0;
pub const SQL_SC_NON_UNIQUE: u32 = 0;
pub const SQL_SC_TRY_UNIQUE: u32 = 1;
pub const SQL_SC_UNIQUE: u32 = 2;
pub const SQL_RD_OFF: u32 = 0;
pub const SQL_RD_ON: u32 = 1;
pub const SQL_RD_DEFAULT: u32 = 1;
pub const SQL_UB_OFF: u32 = 0;
pub const SQL_UB_ON: u32 = 1;
pub const SQL_UB_DEFAULT: u32 = 0;
pub const SQL_UB_FIXED: u32 = 1;
pub const SQL_UB_VARIABLE: u32 = 2;
pub const SQL_DESC_ARRAY_SIZE: u32 = 20;
pub const SQL_DESC_ARRAY_STATUS_PTR: u32 = 21;
pub const SQL_DESC_BASE_COLUMN_NAME: u32 = 22;
pub const SQL_DESC_BASE_TABLE_NAME: u32 = 23;
pub const SQL_DESC_BIND_OFFSET_PTR: u32 = 24;
pub const SQL_DESC_BIND_TYPE: u32 = 25;
pub const SQL_DESC_DATETIME_INTERVAL_PRECISION: u32 = 26;
pub const SQL_DESC_LITERAL_PREFIX: u32 = 27;
pub const SQL_DESC_LITERAL_SUFFIX: u32 = 28;
pub const SQL_DESC_LOCAL_TYPE_NAME: u32 = 29;
pub const SQL_DESC_MAXIMUM_SCALE: u32 = 30;
pub const SQL_DESC_MINIMUM_SCALE: u32 = 31;
pub const SQL_DESC_NUM_PREC_RADIX: u32 = 32;
pub const SQL_DESC_PARAMETER_TYPE: u32 = 33;
pub const SQL_DESC_ROWS_PROCESSED_PTR: u32 = 34;
pub const SQL_DESC_ROWVER: u32 = 35;
pub const SQL_DIAG_CURSOR_ROW_COUNT: i32 = -1249;
pub const SQL_DIAG_ROW_NUMBER: i32 = -1248;
pub const SQL_DIAG_COLUMN_NUMBER: i32 = -1247;
pub const SQL_DATE: u32 = 9;
pub const SQL_INTERVAL: u32 = 10;
pub const SQL_TIME: u32 = 10;
pub const SQL_TIMESTAMP: u32 = 11;
pub const SQL_LONGVARCHAR: i32 = -1;
pub const SQL_BINARY: i32 = -2;
pub const SQL_VARBINARY: i32 = -3;
pub const SQL_LONGVARBINARY: i32 = -4;
pub const SQL_BIGINT: i32 = -5;
pub const SQL_TINYINT: i32 = -6;
pub const SQL_BIT: i32 = -7;
pub const SQL_GUID: i32 = -11;
pub const SQL_CODE_YEAR: u32 = 1;
pub const SQL_CODE_MONTH: u32 = 2;
pub const SQL_CODE_DAY: u32 = 3;
pub const SQL_CODE_HOUR: u32 = 4;
pub const SQL_CODE_MINUTE: u32 = 5;
pub const SQL_CODE_SECOND: u32 = 6;
pub const SQL_CODE_YEAR_TO_MONTH: u32 = 7;
pub const SQL_CODE_DAY_TO_HOUR: u32 = 8;
pub const SQL_CODE_DAY_TO_MINUTE: u32 = 9;
pub const SQL_CODE_DAY_TO_SECOND: u32 = 10;
pub const SQL_CODE_HOUR_TO_MINUTE: u32 = 11;
pub const SQL_CODE_HOUR_TO_SECOND: u32 = 12;
pub const SQL_CODE_MINUTE_TO_SECOND: u32 = 13;
pub const SQL_INTERVAL_YEAR: u32 = 101;
pub const SQL_INTERVAL_MONTH: u32 = 102;
pub const SQL_INTERVAL_DAY: u32 = 103;
pub const SQL_INTERVAL_HOUR: u32 = 104;
pub const SQL_INTERVAL_MINUTE: u32 = 105;
pub const SQL_INTERVAL_SECOND: u32 = 106;
pub const SQL_INTERVAL_YEAR_TO_MONTH: u32 = 107;
pub const SQL_INTERVAL_DAY_TO_HOUR: u32 = 108;
pub const SQL_INTERVAL_DAY_TO_MINUTE: u32 = 109;
pub const SQL_INTERVAL_DAY_TO_SECOND: u32 = 110;
pub const SQL_INTERVAL_HOUR_TO_MINUTE: u32 = 111;
pub const SQL_INTERVAL_HOUR_TO_SECOND: u32 = 112;
pub const SQL_INTERVAL_MINUTE_TO_SECOND: u32 = 113;
pub const SQL_UNICODE: i32 = -8;
pub const SQL_UNICODE_VARCHAR: i32 = -9;
pub const SQL_UNICODE_LONGVARCHAR: i32 = -10;
pub const SQL_UNICODE_CHAR: i32 = -8;
pub const SQL_C_CHAR: u32 = 1;
pub const SQL_C_LONG: u32 = 4;
pub const SQL_C_SHORT: u32 = 5;
pub const SQL_C_FLOAT: u32 = 7;
pub const SQL_C_DOUBLE: u32 = 8;
pub const SQL_C_NUMERIC: u32 = 2;
pub const SQL_C_DEFAULT: u32 = 99;
pub const SQL_SIGNED_OFFSET: i32 = -20;
pub const SQL_UNSIGNED_OFFSET: i32 = -22;
pub const SQL_C_DATE: u32 = 9;
pub const SQL_C_TIME: u32 = 10;
pub const SQL_C_TIMESTAMP: u32 = 11;
pub const SQL_C_TYPE_DATE: u32 = 91;
pub const SQL_C_TYPE_TIME: u32 = 92;
pub const SQL_C_TYPE_TIMESTAMP: u32 = 93;
pub const SQL_C_INTERVAL_YEAR: u32 = 101;
pub const SQL_C_INTERVAL_MONTH: u32 = 102;
pub const SQL_C_INTERVAL_DAY: u32 = 103;
pub const SQL_C_INTERVAL_HOUR: u32 = 104;
pub const SQL_C_INTERVAL_MINUTE: u32 = 105;
pub const SQL_C_INTERVAL_SECOND: u32 = 106;
pub const SQL_C_INTERVAL_YEAR_TO_MONTH: u32 = 107;
pub const SQL_C_INTERVAL_DAY_TO_HOUR: u32 = 108;
pub const SQL_C_INTERVAL_DAY_TO_MINUTE: u32 = 109;
pub const SQL_C_INTERVAL_DAY_TO_SECOND: u32 = 110;
pub const SQL_C_INTERVAL_HOUR_TO_MINUTE: u32 = 111;
pub const SQL_C_INTERVAL_HOUR_TO_SECOND: u32 = 112;
pub const SQL_C_INTERVAL_MINUTE_TO_SECOND: u32 = 113;
pub const SQL_C_BINARY: i32 = -2;
pub const SQL_C_BIT: i32 = -7;
pub const SQL_C_SBIGINT: i32 = -25;
pub const SQL_C_UBIGINT: i32 = -27;
pub const SQL_C_TINYINT: i32 = -6;
pub const SQL_C_SLONG: i32 = -16;
pub const SQL_C_SSHORT: i32 = -15;
pub const SQL_C_STINYINT: i32 = -26;
pub const SQL_C_ULONG: i32 = -18;
pub const SQL_C_USHORT: i32 = -17;
pub const SQL_C_UTINYINT: i32 = -28;
pub const SQL_C_BOOKMARK: i32 = -18;
pub const SQL_C_GUID: i32 = -11;
pub const SQL_TYPE_NULL: u32 = 0;
pub const SQL_DRIVER_C_TYPE_BASE: u32 = 16384;
pub const SQL_DRIVER_SQL_TYPE_BASE: u32 = 16384;
pub const SQL_DRIVER_DESC_FIELD_BASE: u32 = 16384;
pub const SQL_DRIVER_DIAG_FIELD_BASE: u32 = 16384;
pub const SQL_DRIVER_INFO_TYPE_BASE: u32 = 16384;
pub const SQL_DRIVER_CONN_ATTR_BASE: u32 = 16384;
pub const SQL_DRIVER_STMT_ATTR_BASE: u32 = 16384;
pub const SQL_C_VARBOOKMARK: i32 = -2;
pub const SQL_NO_ROW_NUMBER: i32 = -1;
pub const SQL_NO_COLUMN_NUMBER: i32 = -1;
pub const SQL_DEFAULT_PARAM: i32 = -5;
pub const SQL_IGNORE: i32 = -6;
pub const SQL_COLUMN_IGNORE: i32 = -6;
pub const SQL_LEN_DATA_AT_EXEC_OFFSET: i32 = -100;
pub const SQL_LEN_BINARY_ATTR_OFFSET: i32 = -100;
pub const SQL_SETPARAM_VALUE_MAX: i32 = -1;
pub const SQL_COLUMN_COUNT: u32 = 0;
pub const SQL_COLUMN_NAME: u32 = 1;
pub const SQL_COLUMN_TYPE: u32 = 2;
pub const SQL_COLUMN_LENGTH: u32 = 3;
pub const SQL_COLUMN_PRECISION: u32 = 4;
pub const SQL_COLUMN_SCALE: u32 = 5;
pub const SQL_COLUMN_DISPLAY_SIZE: u32 = 6;
pub const SQL_COLUMN_NULLABLE: u32 = 7;
pub const SQL_COLUMN_UNSIGNED: u32 = 8;
pub const SQL_COLUMN_MONEY: u32 = 9;
pub const SQL_COLUMN_UPDATABLE: u32 = 10;
pub const SQL_COLUMN_AUTO_INCREMENT: u32 = 11;
pub const SQL_COLUMN_CASE_SENSITIVE: u32 = 12;
pub const SQL_COLUMN_SEARCHABLE: u32 = 13;
pub const SQL_COLUMN_TYPE_NAME: u32 = 14;
pub const SQL_COLUMN_TABLE_NAME: u32 = 15;
pub const SQL_COLUMN_OWNER_NAME: u32 = 16;
pub const SQL_COLUMN_QUALIFIER_NAME: u32 = 17;
pub const SQL_COLUMN_LABEL: u32 = 18;
pub const SQL_COLATT_OPT_MAX: u32 = 18;
pub const SQL_COLATT_OPT_MIN: u32 = 0;
pub const SQL_ATTR_READONLY: u32 = 0;
pub const SQL_ATTR_WRITE: u32 = 1;
pub const SQL_ATTR_READWRITE_UNKNOWN: u32 = 2;
pub const SQL_UNSEARCHABLE: u32 = 0;
pub const SQL_LIKE_ONLY: u32 = 1;
pub const SQL_ALL_EXCEPT_LIKE: u32 = 2;
pub const SQL_SEARCHABLE: u32 = 3;
pub const SQL_PRED_SEARCHABLE: u32 = 3;
pub const SQL_NO_TOTAL: i32 = -4;
pub const SQL_API_SQLALLOCHANDLESTD: u32 = 73;
pub const SQL_API_SQLBULKOPERATIONS: u32 = 24;
pub const SQL_API_SQLBINDPARAMETER: u32 = 72;
pub const SQL_API_SQLBROWSECONNECT: u32 = 55;
pub const SQL_API_SQLCOLATTRIBUTES: u32 = 6;
pub const SQL_API_SQLCOLUMNPRIVILEGES: u32 = 56;
pub const SQL_API_SQLDESCRIBEPARAM: u32 = 58;
pub const SQL_API_SQLDRIVERCONNECT: u32 = 41;
pub const SQL_API_SQLDRIVERS: u32 = 71;
pub const SQL_API_SQLEXTENDEDFETCH: u32 = 59;
pub const SQL_API_SQLFOREIGNKEYS: u32 = 60;
pub const SQL_API_SQLMORERESULTS: u32 = 61;
pub const SQL_API_SQLNATIVESQL: u32 = 62;
pub const SQL_API_SQLNUMPARAMS: u32 = 63;
pub const SQL_API_SQLPARAMOPTIONS: u32 = 64;
pub const SQL_API_SQLPRIMARYKEYS: u32 = 65;
pub const SQL_API_SQLPROCEDURECOLUMNS: u32 = 66;
pub const SQL_API_SQLPROCEDURES: u32 = 67;
pub const SQL_API_SQLSETPOS: u32 = 68;
pub const SQL_API_SQLSETSCROLLOPTIONS: u32 = 69;
pub const SQL_API_SQLTABLEPRIVILEGES: u32 = 70;
pub const SQL_API_ALL_FUNCTIONS: u32 = 0;
pub const SQL_API_LOADBYORDINAL: u32 = 199;
pub const SQL_API_ODBC3_ALL_FUNCTIONS: u32 = 999;
pub const SQL_API_ODBC3_ALL_FUNCTIONS_SIZE: u32 = 250;
pub const SQL_INFO_FIRST: u32 = 0;
pub const SQL_ACTIVE_CONNECTIONS: u32 = 0;
pub const SQL_ACTIVE_STATEMENTS: u32 = 1;
pub const SQL_DRIVER_HDBC: u32 = 3;
pub const SQL_DRIVER_HENV: u32 = 4;
pub const SQL_DRIVER_HSTMT: u32 = 5;
pub const SQL_DRIVER_NAME: u32 = 6;
pub const SQL_DRIVER_VER: u32 = 7;
pub const SQL_ODBC_API_CONFORMANCE: u32 = 9;
pub const SQL_ODBC_VER: u32 = 10;
pub const SQL_ROW_UPDATES: u32 = 11;
pub const SQL_ODBC_SAG_CLI_CONFORMANCE: u32 = 12;
pub const SQL_ODBC_SQL_CONFORMANCE: u32 = 15;
pub const SQL_PROCEDURES: u32 = 21;
pub const SQL_CONCAT_NULL_BEHAVIOR: u32 = 22;
pub const SQL_CURSOR_ROLLBACK_BEHAVIOR: u32 = 24;
pub const SQL_EXPRESSIONS_IN_ORDERBY: u32 = 27;
pub const SQL_MAX_OWNER_NAME_LEN: u32 = 32;
pub const SQL_MAX_PROCEDURE_NAME_LEN: u32 = 33;
pub const SQL_MAX_QUALIFIER_NAME_LEN: u32 = 34;
pub const SQL_MULT_RESULT_SETS: u32 = 36;
pub const SQL_MULTIPLE_ACTIVE_TXN: u32 = 37;
pub const SQL_OUTER_JOINS: u32 = 38;
pub const SQL_OWNER_TERM: u32 = 39;
pub const SQL_PROCEDURE_TERM: u32 = 40;
pub const SQL_QUALIFIER_NAME_SEPARATOR: u32 = 41;
pub const SQL_QUALIFIER_TERM: u32 = 42;
pub const SQL_SCROLL_OPTIONS: u32 = 44;
pub const SQL_TABLE_TERM: u32 = 45;
pub const SQL_CONVERT_FUNCTIONS: u32 = 48;
pub const SQL_NUMERIC_FUNCTIONS: u32 = 49;
pub const SQL_STRING_FUNCTIONS: u32 = 50;
pub const SQL_SYSTEM_FUNCTIONS: u32 = 51;
pub const SQL_TIMEDATE_FUNCTIONS: u32 = 52;
pub const SQL_CONVERT_BIGINT: u32 = 53;
pub const SQL_CONVERT_BINARY: u32 = 54;
pub const SQL_CONVERT_BIT: u32 = 55;
pub const SQL_CONVERT_CHAR: u32 = 56;
pub const SQL_CONVERT_DATE: u32 = 57;
pub const SQL_CONVERT_DECIMAL: u32 = 58;
pub const SQL_CONVERT_DOUBLE: u32 = 59;
pub const SQL_CONVERT_FLOAT: u32 = 60;
pub const SQL_CONVERT_INTEGER: u32 = 61;
pub const SQL_CONVERT_LONGVARCHAR: u32 = 62;
pub const SQL_CONVERT_NUMERIC: u32 = 63;
pub const SQL_CONVERT_REAL: u32 = 64;
pub const SQL_CONVERT_SMALLINT: u32 = 65;
pub const SQL_CONVERT_TIME: u32 = 66;
pub const SQL_CONVERT_TIMESTAMP: u32 = 67;
pub const SQL_CONVERT_TINYINT: u32 = 68;
pub const SQL_CONVERT_VARBINARY: u32 = 69;
pub const SQL_CONVERT_VARCHAR: u32 = 70;
pub const SQL_CONVERT_LONGVARBINARY: u32 = 71;
pub const SQL_ODBC_SQL_OPT_IEF: u32 = 73;
pub const SQL_CORRELATION_NAME: u32 = 74;
pub const SQL_NON_NULLABLE_COLUMNS: u32 = 75;
pub const SQL_DRIVER_HLIB: u32 = 76;
pub const SQL_DRIVER_ODBC_VER: u32 = 77;
pub const SQL_LOCK_TYPES: u32 = 78;
pub const SQL_POS_OPERATIONS: u32 = 79;
pub const SQL_POSITIONED_STATEMENTS: u32 = 80;
pub const SQL_BOOKMARK_PERSISTENCE: u32 = 82;
pub const SQL_STATIC_SENSITIVITY: u32 = 83;
pub const SQL_FILE_USAGE: u32 = 84;
pub const SQL_COLUMN_ALIAS: u32 = 87;
pub const SQL_GROUP_BY: u32 = 88;
pub const SQL_KEYWORDS: u32 = 89;
pub const SQL_OWNER_USAGE: u32 = 91;
pub const SQL_QUALIFIER_USAGE: u32 = 92;
pub const SQL_QUOTED_IDENTIFIER_CASE: u32 = 93;
pub const SQL_SUBQUERIES: u32 = 95;
pub const SQL_UNION: u32 = 96;
pub const SQL_MAX_ROW_SIZE_INCLUDES_LONG: u32 = 103;
pub const SQL_MAX_CHAR_LITERAL_LEN: u32 = 108;
pub const SQL_TIMEDATE_ADD_INTERVALS: u32 = 109;
pub const SQL_TIMEDATE_DIFF_INTERVALS: u32 = 110;
pub const SQL_NEED_LONG_DATA_LEN: u32 = 111;
pub const SQL_MAX_BINARY_LITERAL_LEN: u32 = 112;
pub const SQL_LIKE_ESCAPE_CLAUSE: u32 = 113;
pub const SQL_QUALIFIER_LOCATION: u32 = 114;
pub const SQL_ACTIVE_ENVIRONMENTS: u32 = 116;
pub const SQL_ALTER_DOMAIN: u32 = 117;
pub const SQL_SQL_CONFORMANCE: u32 = 118;
pub const SQL_DATETIME_LITERALS: u32 = 119;
pub const SQL_ASYNC_MODE: u32 = 10021;
pub const SQL_BATCH_ROW_COUNT: u32 = 120;
pub const SQL_BATCH_SUPPORT: u32 = 121;
pub const SQL_CATALOG_LOCATION: u32 = 114;
pub const SQL_CATALOG_NAME_SEPARATOR: u32 = 41;
pub const SQL_CATALOG_TERM: u32 = 42;
pub const SQL_CATALOG_USAGE: u32 = 92;
pub const SQL_CONVERT_WCHAR: u32 = 122;
pub const SQL_CONVERT_INTERVAL_DAY_TIME: u32 = 123;
pub const SQL_CONVERT_INTERVAL_YEAR_MONTH: u32 = 124;
pub const SQL_CONVERT_WLONGVARCHAR: u32 = 125;
pub const SQL_CONVERT_WVARCHAR: u32 = 126;
pub const SQL_CREATE_ASSERTION: u32 = 127;
pub const SQL_CREATE_CHARACTER_SET: u32 = 128;
pub const SQL_CREATE_COLLATION: u32 = 129;
pub const SQL_CREATE_DOMAIN: u32 = 130;
pub const SQL_CREATE_SCHEMA: u32 = 131;
pub const SQL_CREATE_TABLE: u32 = 132;
pub const SQL_CREATE_TRANSLATION: u32 = 133;
pub const SQL_CREATE_VIEW: u32 = 134;
pub const SQL_DRIVER_HDESC: u32 = 135;
pub const SQL_DROP_ASSERTION: u32 = 136;
pub const SQL_DROP_CHARACTER_SET: u32 = 137;
pub const SQL_DROP_COLLATION: u32 = 138;
pub const SQL_DROP_DOMAIN: u32 = 139;
pub const SQL_DROP_SCHEMA: u32 = 140;
pub const SQL_DROP_TABLE: u32 = 141;
pub const SQL_DROP_TRANSLATION: u32 = 142;
pub const SQL_DROP_VIEW: u32 = 143;
pub const SQL_DYNAMIC_CURSOR_ATTRIBUTES1: u32 = 144;
pub const SQL_DYNAMIC_CURSOR_ATTRIBUTES2: u32 = 145;
pub const SQL_FORWARD_ONLY_CURSOR_ATTRIBUTES1: u32 = 146;
pub const SQL_FORWARD_ONLY_CURSOR_ATTRIBUTES2: u32 = 147;
pub const SQL_INDEX_KEYWORDS: u32 = 148;
pub const SQL_INFO_SCHEMA_VIEWS: u32 = 149;
pub const SQL_KEYSET_CURSOR_ATTRIBUTES1: u32 = 150;
pub const SQL_KEYSET_CURSOR_ATTRIBUTES2: u32 = 151;
pub const SQL_MAX_ASYNC_CONCURRENT_STATEMENTS: u32 = 10022;
pub const SQL_ODBC_INTERFACE_CONFORMANCE: u32 = 152;
pub const SQL_PARAM_ARRAY_ROW_COUNTS: u32 = 153;
pub const SQL_PARAM_ARRAY_SELECTS: u32 = 154;
pub const SQL_SCHEMA_TERM: u32 = 39;
pub const SQL_SCHEMA_USAGE: u32 = 91;
pub const SQL_SQL92_DATETIME_FUNCTIONS: u32 = 155;
pub const SQL_SQL92_FOREIGN_KEY_DELETE_RULE: u32 = 156;
pub const SQL_SQL92_FOREIGN_KEY_UPDATE_RULE: u32 = 157;
pub const SQL_SQL92_GRANT: u32 = 158;
pub const SQL_SQL92_NUMERIC_VALUE_FUNCTIONS: u32 = 159;
pub const SQL_SQL92_PREDICATES: u32 = 160;
pub const SQL_SQL92_RELATIONAL_JOIN_OPERATORS: u32 = 161;
pub const SQL_SQL92_REVOKE: u32 = 162;
pub const SQL_SQL92_ROW_VALUE_CONSTRUCTOR: u32 = 163;
pub const SQL_SQL92_STRING_FUNCTIONS: u32 = 164;
pub const SQL_SQL92_VALUE_EXPRESSIONS: u32 = 165;
pub const SQL_STANDARD_CLI_CONFORMANCE: u32 = 166;
pub const SQL_STATIC_CURSOR_ATTRIBUTES1: u32 = 167;
pub const SQL_STATIC_CURSOR_ATTRIBUTES2: u32 = 168;
pub const SQL_AGGREGATE_FUNCTIONS: u32 = 169;
pub const SQL_DDL_INDEX: u32 = 170;
pub const SQL_DM_VER: u32 = 171;
pub const SQL_INSERT_STATEMENT: u32 = 172;
pub const SQL_CONVERT_GUID: u32 = 173;
pub const SQL_UNION_STATEMENT: u32 = 96;
pub const SQL_ASYNC_DBC_FUNCTIONS: u32 = 10023;
pub const SQL_DTC_TRANSITION_COST: u32 = 1750;
pub const SQL_AT_ADD_COLUMN_SINGLE: u32 = 32;
pub const SQL_AT_ADD_COLUMN_DEFAULT: u32 = 64;
pub const SQL_AT_ADD_COLUMN_COLLATION: u32 = 128;
pub const SQL_AT_SET_COLUMN_DEFAULT: u32 = 256;
pub const SQL_AT_DROP_COLUMN_DEFAULT: u32 = 512;
pub const SQL_AT_DROP_COLUMN_CASCADE: u32 = 1024;
pub const SQL_AT_DROP_COLUMN_RESTRICT: u32 = 2048;
pub const SQL_AT_ADD_TABLE_CONSTRAINT: u32 = 4096;
pub const SQL_AT_DROP_TABLE_CONSTRAINT_CASCADE: u32 = 8192;
pub const SQL_AT_DROP_TABLE_CONSTRAINT_RESTRICT: u32 = 16384;
pub const SQL_AT_CONSTRAINT_NAME_DEFINITION: u32 = 32768;
pub const SQL_AT_CONSTRAINT_INITIALLY_DEFERRED: u32 = 65536;
pub const SQL_AT_CONSTRAINT_INITIALLY_IMMEDIATE: u32 = 131072;
pub const SQL_AT_CONSTRAINT_DEFERRABLE: u32 = 262144;
pub const SQL_AT_CONSTRAINT_NON_DEFERRABLE: u32 = 524288;
pub const SQL_CVT_CHAR: u32 = 1;
pub const SQL_CVT_NUMERIC: u32 = 2;
pub const SQL_CVT_DECIMAL: u32 = 4;
pub const SQL_CVT_INTEGER: u32 = 8;
pub const SQL_CVT_SMALLINT: u32 = 16;
pub const SQL_CVT_FLOAT: u32 = 32;
pub const SQL_CVT_REAL: u32 = 64;
pub const SQL_CVT_DOUBLE: u32 = 128;
pub const SQL_CVT_VARCHAR: u32 = 256;
pub const SQL_CVT_LONGVARCHAR: u32 = 512;
pub const SQL_CVT_BINARY: u32 = 1024;
pub const SQL_CVT_VARBINARY: u32 = 2048;
pub const SQL_CVT_BIT: u32 = 4096;
pub const SQL_CVT_TINYINT: u32 = 8192;
pub const SQL_CVT_BIGINT: u32 = 16384;
pub const SQL_CVT_DATE: u32 = 32768;
pub const SQL_CVT_TIME: u32 = 65536;
pub const SQL_CVT_TIMESTAMP: u32 = 131072;
pub const SQL_CVT_LONGVARBINARY: u32 = 262144;
pub const SQL_CVT_INTERVAL_YEAR_MONTH: u32 = 524288;
pub const SQL_CVT_INTERVAL_DAY_TIME: u32 = 1048576;
pub const SQL_CVT_WCHAR: u32 = 2097152;
pub const SQL_CVT_WLONGVARCHAR: u32 = 4194304;
pub const SQL_CVT_WVARCHAR: u32 = 8388608;
pub const SQL_CVT_GUID: u32 = 16777216;
pub const SQL_FN_CVT_CONVERT: u32 = 1;
pub const SQL_FN_CVT_CAST: u32 = 2;
pub const SQL_FN_STR_CONCAT: u32 = 1;
pub const SQL_FN_STR_INSERT: u32 = 2;
pub const SQL_FN_STR_LEFT: u32 = 4;
pub const SQL_FN_STR_LTRIM: u32 = 8;
pub const SQL_FN_STR_LENGTH: u32 = 16;
pub const SQL_FN_STR_LOCATE: u32 = 32;
pub const SQL_FN_STR_LCASE: u32 = 64;
pub const SQL_FN_STR_REPEAT: u32 = 128;
pub const SQL_FN_STR_REPLACE: u32 = 256;
pub const SQL_FN_STR_RIGHT: u32 = 512;
pub const SQL_FN_STR_RTRIM: u32 = 1024;
pub const SQL_FN_STR_SUBSTRING: u32 = 2048;
pub const SQL_FN_STR_UCASE: u32 = 4096;
pub const SQL_FN_STR_ASCII: u32 = 8192;
pub const SQL_FN_STR_CHAR: u32 = 16384;
pub const SQL_FN_STR_DIFFERENCE: u32 = 32768;
pub const SQL_FN_STR_LOCATE_2: u32 = 65536;
pub const SQL_FN_STR_SOUNDEX: u32 = 131072;
pub const SQL_FN_STR_SPACE: u32 = 262144;
pub const SQL_FN_STR_BIT_LENGTH: u32 = 524288;
pub const SQL_FN_STR_CHAR_LENGTH: u32 = 1048576;
pub const SQL_FN_STR_CHARACTER_LENGTH: u32 = 2097152;
pub const SQL_FN_STR_OCTET_LENGTH: u32 = 4194304;
pub const SQL_FN_STR_POSITION: u32 = 8388608;
pub const SQL_SSF_CONVERT: u32 = 1;
pub const SQL_SSF_LOWER: u32 = 2;
pub const SQL_SSF_UPPER: u32 = 4;
pub const SQL_SSF_SUBSTRING: u32 = 8;
pub const SQL_SSF_TRANSLATE: u32 = 16;
pub const SQL_SSF_TRIM_BOTH: u32 = 32;
pub const SQL_SSF_TRIM_LEADING: u32 = 64;
pub const SQL_SSF_TRIM_TRAILING: u32 = 128;
pub const SQL_FN_NUM_ABS: u32 = 1;
pub const SQL_FN_NUM_ACOS: u32 = 2;
pub const SQL_FN_NUM_ASIN: u32 = 4;
pub const SQL_FN_NUM_ATAN: u32 = 8;
pub const SQL_FN_NUM_ATAN2: u32 = 16;
pub const SQL_FN_NUM_CEILING: u32 = 32;
pub const SQL_FN_NUM_COS: u32 = 64;
pub const SQL_FN_NUM_COT: u32 = 128;
pub const SQL_FN_NUM_EXP: u32 = 256;
pub const SQL_FN_NUM_FLOOR: u32 = 512;
pub const SQL_FN_NUM_LOG: u32 = 1024;
pub const SQL_FN_NUM_MOD: u32 = 2048;
pub const SQL_FN_NUM_SIGN: u32 = 4096;
pub const SQL_FN_NUM_SIN: u32 = 8192;
pub const SQL_FN_NUM_SQRT: u32 = 16384;
pub const SQL_FN_NUM_TAN: u32 = 32768;
pub const SQL_FN_NUM_PI: u32 = 65536;
pub const SQL_FN_NUM_RAND: u32 = 131072;
pub const SQL_FN_NUM_DEGREES: u32 = 262144;
pub const SQL_FN_NUM_LOG10: u32 = 524288;
pub const SQL_FN_NUM_POWER: u32 = 1048576;
pub const SQL_FN_NUM_RADIANS: u32 = 2097152;
pub const SQL_FN_NUM_ROUND: u32 = 4194304;
pub const SQL_FN_NUM_TRUNCATE: u32 = 8388608;
pub const SQL_SNVF_BIT_LENGTH: u32 = 1;
pub const SQL_SNVF_CHAR_LENGTH: u32 = 2;
pub const SQL_SNVF_CHARACTER_LENGTH: u32 = 4;
pub const SQL_SNVF_EXTRACT: u32 = 8;
pub const SQL_SNVF_OCTET_LENGTH: u32 = 16;
pub const SQL_SNVF_POSITION: u32 = 32;
pub const SQL_FN_TD_NOW: u32 = 1;
pub const SQL_FN_TD_CURDATE: u32 = 2;
pub const SQL_FN_TD_DAYOFMONTH: u32 = 4;
pub const SQL_FN_TD_DAYOFWEEK: u32 = 8;
pub const SQL_FN_TD_DAYOFYEAR: u32 = 16;
pub const SQL_FN_TD_MONTH: u32 = 32;
pub const SQL_FN_TD_QUARTER: u32 = 64;
pub const SQL_FN_TD_WEEK: u32 = 128;
pub const SQL_FN_TD_YEAR: u32 = 256;
pub const SQL_FN_TD_CURTIME: u32 = 512;
pub const SQL_FN_TD_HOUR: u32 = 1024;
pub const SQL_FN_TD_MINUTE: u32 = 2048;
pub const SQL_FN_TD_SECOND: u32 = 4096;
pub const SQL_FN_TD_TIMESTAMPADD: u32 = 8192;
pub const SQL_FN_TD_TIMESTAMPDIFF: u32 = 16384;
pub const SQL_FN_TD_DAYNAME: u32 = 32768;
pub const SQL_FN_TD_MONTHNAME: u32 = 65536;
pub const SQL_FN_TD_CURRENT_DATE: u32 = 131072;
pub const SQL_FN_TD_CURRENT_TIME: u32 = 262144;
pub const SQL_FN_TD_CURRENT_TIMESTAMP: u32 = 524288;
pub const SQL_FN_TD_EXTRACT: u32 = 1048576;
pub const SQL_SDF_CURRENT_DATE: u32 = 1;
pub const SQL_SDF_CURRENT_TIME: u32 = 2;
pub const SQL_SDF_CURRENT_TIMESTAMP: u32 = 4;
pub const SQL_FN_SYS_USERNAME: u32 = 1;
pub const SQL_FN_SYS_DBNAME: u32 = 2;
pub const SQL_FN_SYS_IFNULL: u32 = 4;
pub const SQL_FN_TSI_FRAC_SECOND: u32 = 1;
pub const SQL_FN_TSI_SECOND: u32 = 2;
pub const SQL_FN_TSI_MINUTE: u32 = 4;
pub const SQL_FN_TSI_HOUR: u32 = 8;
pub const SQL_FN_TSI_DAY: u32 = 16;
pub const SQL_FN_TSI_WEEK: u32 = 32;
pub const SQL_FN_TSI_MONTH: u32 = 64;
pub const SQL_FN_TSI_QUARTER: u32 = 128;
pub const SQL_FN_TSI_YEAR: u32 = 256;
pub const SQL_CA1_NEXT: u32 = 1;
pub const SQL_CA1_ABSOLUTE: u32 = 2;
pub const SQL_CA1_RELATIVE: u32 = 4;
pub const SQL_CA1_BOOKMARK: u32 = 8;
pub const SQL_CA1_LOCK_NO_CHANGE: u32 = 64;
pub const SQL_CA1_LOCK_EXCLUSIVE: u32 = 128;
pub const SQL_CA1_LOCK_UNLOCK: u32 = 256;
pub const SQL_CA1_POS_POSITION: u32 = 512;
pub const SQL_CA1_POS_UPDATE: u32 = 1024;
pub const SQL_CA1_POS_DELETE: u32 = 2048;
pub const SQL_CA1_POS_REFRESH: u32 = 4096;
pub const SQL_CA1_POSITIONED_UPDATE: u32 = 8192;
pub const SQL_CA1_POSITIONED_DELETE: u32 = 16384;
pub const SQL_CA1_SELECT_FOR_UPDATE: u32 = 32768;
pub const SQL_CA1_BULK_ADD: u32 = 65536;
pub const SQL_CA1_BULK_UPDATE_BY_BOOKMARK: u32 = 131072;
pub const SQL_CA1_BULK_DELETE_BY_BOOKMARK: u32 = 262144;
pub const SQL_CA1_BULK_FETCH_BY_BOOKMARK: u32 = 524288;
pub const SQL_CA2_READ_ONLY_CONCURRENCY: u32 = 1;
pub const SQL_CA2_LOCK_CONCURRENCY: u32 = 2;
pub const SQL_CA2_OPT_ROWVER_CONCURRENCY: u32 = 4;
pub const SQL_CA2_OPT_VALUES_CONCURRENCY: u32 = 8;
pub const SQL_CA2_SENSITIVITY_ADDITIONS: u32 = 16;
pub const SQL_CA2_SENSITIVITY_DELETIONS: u32 = 32;
pub const SQL_CA2_SENSITIVITY_UPDATES: u32 = 64;
pub const SQL_CA2_MAX_ROWS_SELECT: u32 = 128;
pub const SQL_CA2_MAX_ROWS_INSERT: u32 = 256;
pub const SQL_CA2_MAX_ROWS_DELETE: u32 = 512;
pub const SQL_CA2_MAX_ROWS_UPDATE: u32 = 1024;
pub const SQL_CA2_MAX_ROWS_CATALOG: u32 = 2048;
pub const SQL_CA2_MAX_ROWS_AFFECTS_ALL: u32 = 3968;
pub const SQL_CA2_CRC_EXACT: u32 = 4096;
pub const SQL_CA2_CRC_APPROXIMATE: u32 = 8192;
pub const SQL_CA2_SIMULATE_NON_UNIQUE: u32 = 16384;
pub const SQL_CA2_SIMULATE_TRY_UNIQUE: u32 = 32768;
pub const SQL_CA2_SIMULATE_UNIQUE: u32 = 65536;
pub const SQL_OAC_NONE: u32 = 0;
pub const SQL_OAC_LEVEL1: u32 = 1;
pub const SQL_OAC_LEVEL2: u32 = 2;
pub const SQL_OSCC_NOT_COMPLIANT: u32 = 0;
pub const SQL_OSCC_COMPLIANT: u32 = 1;
pub const SQL_OSC_MINIMUM: u32 = 0;
pub const SQL_OSC_CORE: u32 = 1;
pub const SQL_OSC_EXTENDED: u32 = 2;
pub const SQL_CB_NULL: u32 = 0;
pub const SQL_CB_NON_NULL: u32 = 1;
pub const SQL_SO_FORWARD_ONLY: u32 = 1;
pub const SQL_SO_KEYSET_DRIVEN: u32 = 2;
pub const SQL_SO_DYNAMIC: u32 = 4;
pub const SQL_SO_MIXED: u32 = 8;
pub const SQL_SO_STATIC: u32 = 16;
pub const SQL_FD_FETCH_BOOKMARK: u32 = 128;
pub const SQL_CN_NONE: u32 = 0;
pub const SQL_CN_DIFFERENT: u32 = 1;
pub const SQL_CN_ANY: u32 = 2;
pub const SQL_NNC_NULL: u32 = 0;
pub const SQL_NNC_NON_NULL: u32 = 1;
pub const SQL_NC_START: u32 = 2;
pub const SQL_NC_END: u32 = 4;
pub const SQL_FILE_NOT_SUPPORTED: u32 = 0;
pub const SQL_FILE_TABLE: u32 = 1;
pub const SQL_FILE_QUALIFIER: u32 = 2;
pub const SQL_FILE_CATALOG: u32 = 2;
pub const SQL_GD_BLOCK: u32 = 4;
pub const SQL_GD_BOUND: u32 = 8;
pub const SQL_GD_OUTPUT_PARAMS: u32 = 16;
pub const SQL_PS_POSITIONED_DELETE: u32 = 1;
pub const SQL_PS_POSITIONED_UPDATE: u32 = 2;
pub const SQL_PS_SELECT_FOR_UPDATE: u32 = 4;
pub const SQL_GB_NOT_SUPPORTED: u32 = 0;
pub const SQL_GB_GROUP_BY_EQUALS_SELECT: u32 = 1;
pub const SQL_GB_GROUP_BY_CONTAINS_SELECT: u32 = 2;
pub const SQL_GB_NO_RELATION: u32 = 3;
pub const SQL_GB_COLLATE: u32 = 4;
pub const SQL_OU_DML_STATEMENTS: u32 = 1;
pub const SQL_OU_PROCEDURE_INVOCATION: u32 = 2;
pub const SQL_OU_TABLE_DEFINITION: u32 = 4;
pub const SQL_OU_INDEX_DEFINITION: u32 = 8;
pub const SQL_OU_PRIVILEGE_DEFINITION: u32 = 16;
pub const SQL_SU_DML_STATEMENTS: u32 = 1;
pub const SQL_SU_PROCEDURE_INVOCATION: u32 = 2;
pub const SQL_SU_TABLE_DEFINITION: u32 = 4;
pub const SQL_SU_INDEX_DEFINITION: u32 = 8;
pub const SQL_SU_PRIVILEGE_DEFINITION: u32 = 16;
pub const SQL_QU_DML_STATEMENTS: u32 = 1;
pub const SQL_QU_PROCEDURE_INVOCATION: u32 = 2;
pub const SQL_QU_TABLE_DEFINITION: u32 = 4;
pub const SQL_QU_INDEX_DEFINITION: u32 = 8;
pub const SQL_QU_PRIVILEGE_DEFINITION: u32 = 16;
pub const SQL_CU_DML_STATEMENTS: u32 = 1;
pub const SQL_CU_PROCEDURE_INVOCATION: u32 = 2;
pub const SQL_CU_TABLE_DEFINITION: u32 = 4;
pub const SQL_CU_INDEX_DEFINITION: u32 = 8;
pub const SQL_CU_PRIVILEGE_DEFINITION: u32 = 16;
pub const SQL_SQ_COMPARISON: u32 = 1;
pub const SQL_SQ_EXISTS: u32 = 2;
pub const SQL_SQ_IN: u32 = 4;
pub const SQL_SQ_QUANTIFIED: u32 = 8;
pub const SQL_SQ_CORRELATED_SUBQUERIES: u32 = 16;
pub const SQL_U_UNION: u32 = 1;
pub const SQL_U_UNION_ALL: u32 = 2;
pub const SQL_BP_CLOSE: u32 = 1;
pub const SQL_BP_DELETE: u32 = 2;
pub const SQL_BP_DROP: u32 = 4;
pub const SQL_BP_TRANSACTION: u32 = 8;
pub const SQL_BP_UPDATE: u32 = 16;
pub const SQL_BP_OTHER_HSTMT: u32 = 32;
pub const SQL_BP_SCROLL: u32 = 64;
pub const SQL_SS_ADDITIONS: u32 = 1;
pub const SQL_SS_DELETIONS: u32 = 2;
pub const SQL_SS_UPDATES: u32 = 4;
pub const SQL_CV_CREATE_VIEW: u32 = 1;
pub const SQL_CV_CHECK_OPTION: u32 = 2;
pub const SQL_CV_CASCADED: u32 = 4;
pub const SQL_CV_LOCAL: u32 = 8;
pub const SQL_LCK_NO_CHANGE: u32 = 1;
pub const SQL_LCK_EXCLUSIVE: u32 = 2;
pub const SQL_LCK_UNLOCK: u32 = 4;
pub const SQL_POS_POSITION: u32 = 1;
pub const SQL_POS_REFRESH: u32 = 2;
pub const SQL_POS_UPDATE: u32 = 4;
pub const SQL_POS_DELETE: u32 = 8;
pub const SQL_POS_ADD: u32 = 16;
pub const SQL_QL_START: u32 = 1;
pub const SQL_QL_END: u32 = 2;
pub const SQL_AF_AVG: u32 = 1;
pub const SQL_AF_COUNT: u32 = 2;
pub const SQL_AF_MAX: u32 = 4;
pub const SQL_AF_MIN: u32 = 8;
pub const SQL_AF_SUM: u32 = 16;
pub const SQL_AF_DISTINCT: u32 = 32;
pub const SQL_AF_ALL: u32 = 64;
pub const SQL_SC_SQL92_ENTRY: u32 = 1;
pub const SQL_SC_FIPS127_2_TRANSITIONAL: u32 = 2;
pub const SQL_SC_SQL92_INTERMEDIATE: u32 = 4;
pub const SQL_SC_SQL92_FULL: u32 = 8;
pub const SQL_DL_SQL92_DATE: u32 = 1;
pub const SQL_DL_SQL92_TIME: u32 = 2;
pub const SQL_DL_SQL92_TIMESTAMP: u32 = 4;
pub const SQL_DL_SQL92_INTERVAL_YEAR: u32 = 8;
pub const SQL_DL_SQL92_INTERVAL_MONTH: u32 = 16;
pub const SQL_DL_SQL92_INTERVAL_DAY: u32 = 32;
pub const SQL_DL_SQL92_INTERVAL_HOUR: u32 = 64;
pub const SQL_DL_SQL92_INTERVAL_MINUTE: u32 = 128;
pub const SQL_DL_SQL92_INTERVAL_SECOND: u32 = 256;
pub const SQL_DL_SQL92_INTERVAL_YEAR_TO_MONTH: u32 = 512;
pub const SQL_DL_SQL92_INTERVAL_DAY_TO_HOUR: u32 = 1024;
pub const SQL_DL_SQL92_INTERVAL_DAY_TO_MINUTE: u32 = 2048;
pub const SQL_DL_SQL92_INTERVAL_DAY_TO_SECOND: u32 = 4096;
pub const SQL_DL_SQL92_INTERVAL_HOUR_TO_MINUTE: u32 = 8192;
pub const SQL_DL_SQL92_INTERVAL_HOUR_TO_SECOND: u32 = 16384;
pub const SQL_DL_SQL92_INTERVAL_MINUTE_TO_SECOND: u32 = 32768;
pub const SQL_CL_START: u32 = 1;
pub const SQL_CL_END: u32 = 2;
pub const SQL_BRC_PROCEDURES: u32 = 1;
pub const SQL_BRC_EXPLICIT: u32 = 2;
pub const SQL_BRC_ROLLED_UP: u32 = 4;
pub const SQL_BS_SELECT_EXPLICIT: u32 = 1;
pub const SQL_BS_ROW_COUNT_EXPLICIT: u32 = 2;
pub const SQL_BS_SELECT_PROC: u32 = 4;
pub const SQL_BS_ROW_COUNT_PROC: u32 = 8;
pub const SQL_PARC_BATCH: u32 = 1;
pub const SQL_PARC_NO_BATCH: u32 = 2;
pub const SQL_PAS_BATCH: u32 = 1;
pub const SQL_PAS_NO_BATCH: u32 = 2;
pub const SQL_PAS_NO_SELECT: u32 = 3;
pub const SQL_IK_NONE: u32 = 0;
pub const SQL_IK_ASC: u32 = 1;
pub const SQL_IK_DESC: u32 = 2;
pub const SQL_IK_ALL: u32 = 3;
pub const SQL_ISV_ASSERTIONS: u32 = 1;
pub const SQL_ISV_CHARACTER_SETS: u32 = 2;
pub const SQL_ISV_CHECK_CONSTRAINTS: u32 = 4;
pub const SQL_ISV_COLLATIONS: u32 = 8;
pub const SQL_ISV_COLUMN_DOMAIN_USAGE: u32 = 16;
pub const SQL_ISV_COLUMN_PRIVILEGES: u32 = 32;
pub const SQL_ISV_COLUMNS: u32 = 64;
pub const SQL_ISV_CONSTRAINT_COLUMN_USAGE: u32 = 128;
pub const SQL_ISV_CONSTRAINT_TABLE_USAGE: u32 = 256;
pub const SQL_ISV_DOMAIN_CONSTRAINTS: u32 = 512;
pub const SQL_ISV_DOMAINS: u32 = 1024;
pub const SQL_ISV_KEY_COLUMN_USAGE: u32 = 2048;
pub const SQL_ISV_REFERENTIAL_CONSTRAINTS: u32 = 4096;
pub const SQL_ISV_SCHEMATA: u32 = 8192;
pub const SQL_ISV_SQL_LANGUAGES: u32 = 16384;
pub const SQL_ISV_TABLE_CONSTRAINTS: u32 = 32768;
pub const SQL_ISV_TABLE_PRIVILEGES: u32 = 65536;
pub const SQL_ISV_TABLES: u32 = 131072;
pub const SQL_ISV_TRANSLATIONS: u32 = 262144;
pub const SQL_ISV_USAGE_PRIVILEGES: u32 = 524288;
pub const SQL_ISV_VIEW_COLUMN_USAGE: u32 = 1048576;
pub const SQL_ISV_VIEW_TABLE_USAGE: u32 = 2097152;
pub const SQL_ISV_VIEWS: u32 = 4194304;
pub const SQL_AM_NONE: u32 = 0;
pub const SQL_AM_CONNECTION: u32 = 1;
pub const SQL_AM_STATEMENT: u32 = 2;
pub const SQL_AD_CONSTRAINT_NAME_DEFINITION: u32 = 1;
pub const SQL_AD_ADD_DOMAIN_CONSTRAINT: u32 = 2;
pub const SQL_AD_DROP_DOMAIN_CONSTRAINT: u32 = 4;
pub const SQL_AD_ADD_DOMAIN_DEFAULT: u32 = 8;
pub const SQL_AD_DROP_DOMAIN_DEFAULT: u32 = 16;
pub const SQL_AD_ADD_CONSTRAINT_INITIALLY_DEFERRED: u32 = 32;
pub const SQL_AD_ADD_CONSTRAINT_INITIALLY_IMMEDIATE: u32 = 64;
pub const SQL_AD_ADD_CONSTRAINT_DEFERRABLE: u32 = 128;
pub const SQL_AD_ADD_CONSTRAINT_NON_DEFERRABLE: u32 = 256;
pub const SQL_CS_CREATE_SCHEMA: u32 = 1;
pub const SQL_CS_AUTHORIZATION: u32 = 2;
pub const SQL_CS_DEFAULT_CHARACTER_SET: u32 = 4;
pub const SQL_CTR_CREATE_TRANSLATION: u32 = 1;
pub const SQL_CA_CREATE_ASSERTION: u32 = 1;
pub const SQL_CA_CONSTRAINT_INITIALLY_DEFERRED: u32 = 16;
pub const SQL_CA_CONSTRAINT_INITIALLY_IMMEDIATE: u32 = 32;
pub const SQL_CA_CONSTRAINT_DEFERRABLE: u32 = 64;
pub const SQL_CA_CONSTRAINT_NON_DEFERRABLE: u32 = 128;
pub const SQL_CCS_CREATE_CHARACTER_SET: u32 = 1;
pub const SQL_CCS_COLLATE_CLAUSE: u32 = 2;
pub const SQL_CCS_LIMITED_COLLATION: u32 = 4;
pub const SQL_CCOL_CREATE_COLLATION: u32 = 1;
pub const SQL_CDO_CREATE_DOMAIN: u32 = 1;
pub const SQL_CDO_DEFAULT: u32 = 2;
pub const SQL_CDO_CONSTRAINT: u32 = 4;
pub const SQL_CDO_COLLATION: u32 = 8;
pub const SQL_CDO_CONSTRAINT_NAME_DEFINITION: u32 = 16;
pub const SQL_CDO_CONSTRAINT_INITIALLY_DEFERRED: u32 = 32;
pub const SQL_CDO_CONSTRAINT_INITIALLY_IMMEDIATE: u32 = 64;
pub const SQL_CDO_CONSTRAINT_DEFERRABLE: u32 = 128;
pub const SQL_CDO_CONSTRAINT_NON_DEFERRABLE: u32 = 256;
pub const SQL_CT_CREATE_TABLE: u32 = 1;
pub const SQL_CT_COMMIT_PRESERVE: u32 = 2;
pub const SQL_CT_COMMIT_DELETE: u32 = 4;
pub const SQL_CT_GLOBAL_TEMPORARY: u32 = 8;
pub const SQL_CT_LOCAL_TEMPORARY: u32 = 16;
pub const SQL_CT_CONSTRAINT_INITIALLY_DEFERRED: u32 = 32;
pub const SQL_CT_CONSTRAINT_INITIALLY_IMMEDIATE: u32 = 64;
pub const SQL_CT_CONSTRAINT_DEFERRABLE: u32 = 128;
pub const SQL_CT_CONSTRAINT_NON_DEFERRABLE: u32 = 256;
pub const SQL_CT_COLUMN_CONSTRAINT: u32 = 512;
pub const SQL_CT_COLUMN_DEFAULT: u32 = 1024;
pub const SQL_CT_COLUMN_COLLATION: u32 = 2048;
pub const SQL_CT_TABLE_CONSTRAINT: u32 = 4096;
pub const SQL_CT_CONSTRAINT_NAME_DEFINITION: u32 = 8192;
pub const SQL_DI_CREATE_INDEX: u32 = 1;
pub const SQL_DI_DROP_INDEX: u32 = 2;
pub const SQL_DC_DROP_COLLATION: u32 = 1;
pub const SQL_DD_DROP_DOMAIN: u32 = 1;
pub const SQL_DD_RESTRICT: u32 = 2;
pub const SQL_DD_CASCADE: u32 = 4;
pub const SQL_DS_DROP_SCHEMA: u32 = 1;
pub const SQL_DS_RESTRICT: u32 = 2;
pub const SQL_DS_CASCADE: u32 = 4;
pub const SQL_DCS_DROP_CHARACTER_SET: u32 = 1;
pub const SQL_DA_DROP_ASSERTION: u32 = 1;
pub const SQL_DT_DROP_TABLE: u32 = 1;
pub const SQL_DT_RESTRICT: u32 = 2;
pub const SQL_DT_CASCADE: u32 = 4;
pub const SQL_DTR_DROP_TRANSLATION: u32 = 1;
pub const SQL_DV_DROP_VIEW: u32 = 1;
pub const SQL_DV_RESTRICT: u32 = 2;
pub const SQL_DV_CASCADE: u32 = 4;
pub const SQL_IS_INSERT_LITERALS: u32 = 1;
pub const SQL_IS_INSERT_SEARCHED: u32 = 2;
pub const SQL_IS_SELECT_INTO: u32 = 4;
pub const SQL_OIC_CORE: u32 = 1;
pub const SQL_OIC_LEVEL1: u32 = 2;
pub const SQL_OIC_LEVEL2: u32 = 3;
pub const SQL_SFKD_CASCADE: u32 = 1;
pub const SQL_SFKD_NO_ACTION: u32 = 2;
pub const SQL_SFKD_SET_DEFAULT: u32 = 4;
pub const SQL_SFKD_SET_NULL: u32 = 8;
pub const SQL_SFKU_CASCADE: u32 = 1;
pub const SQL_SFKU_NO_ACTION: u32 = 2;
pub const SQL_SFKU_SET_DEFAULT: u32 = 4;
pub const SQL_SFKU_SET_NULL: u32 = 8;
pub const SQL_SG_USAGE_ON_DOMAIN: u32 = 1;
pub const SQL_SG_USAGE_ON_CHARACTER_SET: u32 = 2;
pub const SQL_SG_USAGE_ON_COLLATION: u32 = 4;
pub const SQL_SG_USAGE_ON_TRANSLATION: u32 = 8;
pub const SQL_SG_WITH_GRANT_OPTION: u32 = 16;
pub const SQL_SG_DELETE_TABLE: u32 = 32;
pub const SQL_SG_INSERT_TABLE: u32 = 64;
pub const SQL_SG_INSERT_COLUMN: u32 = 128;
pub const SQL_SG_REFERENCES_TABLE: u32 = 256;
pub const SQL_SG_REFERENCES_COLUMN: u32 = 512;
pub const SQL_SG_SELECT_TABLE: u32 = 1024;
pub const SQL_SG_UPDATE_TABLE: u32 = 2048;
pub const SQL_SG_UPDATE_COLUMN: u32 = 4096;
pub const SQL_SP_EXISTS: u32 = 1;
pub const SQL_SP_ISNOTNULL: u32 = 2;
pub const SQL_SP_ISNULL: u32 = 4;
pub const SQL_SP_MATCH_FULL: u32 = 8;
pub const SQL_SP_MATCH_PARTIAL: u32 = 16;
pub const SQL_SP_MATCH_UNIQUE_FULL: u32 = 32;
pub const SQL_SP_MATCH_UNIQUE_PARTIAL: u32 = 64;
pub const SQL_SP_OVERLAPS: u32 = 128;
pub const SQL_SP_UNIQUE: u32 = 256;
pub const SQL_SP_LIKE: u32 = 512;
pub const SQL_SP_IN: u32 = 1024;
pub const SQL_SP_BETWEEN: u32 = 2048;
pub const SQL_SP_COMPARISON: u32 = 4096;
pub const SQL_SP_QUANTIFIED_COMPARISON: u32 = 8192;
pub const SQL_SRJO_CORRESPONDING_CLAUSE: u32 = 1;
pub const SQL_SRJO_CROSS_JOIN: u32 = 2;
pub const SQL_SRJO_EXCEPT_JOIN: u32 = 4;
pub const SQL_SRJO_FULL_OUTER_JOIN: u32 = 8;
pub const SQL_SRJO_INNER_JOIN: u32 = 16;
pub const SQL_SRJO_INTERSECT_JOIN: u32 = 32;
pub const SQL_SRJO_LEFT_OUTER_JOIN: u32 = 64;
pub const SQL_SRJO_NATURAL_JOIN: u32 = 128;
pub const SQL_SRJO_RIGHT_OUTER_JOIN: u32 = 256;
pub const SQL_SRJO_UNION_JOIN: u32 = 512;
pub const SQL_SR_USAGE_ON_DOMAIN: u32 = 1;
pub const SQL_SR_USAGE_ON_CHARACTER_SET: u32 = 2;
pub const SQL_SR_USAGE_ON_COLLATION: u32 = 4;
pub const SQL_SR_USAGE_ON_TRANSLATION: u32 = 8;
pub const SQL_SR_GRANT_OPTION_FOR: u32 = 16;
pub const SQL_SR_CASCADE: u32 = 32;
pub const SQL_SR_RESTRICT: u32 = 64;
pub const SQL_SR_DELETE_TABLE: u32 = 128;
pub const SQL_SR_INSERT_TABLE: u32 = 256;
pub const SQL_SR_INSERT_COLUMN: u32 = 512;
pub const SQL_SR_REFERENCES_TABLE: u32 = 1024;
pub const SQL_SR_REFERENCES_COLUMN: u32 = 2048;
pub const SQL_SR_SELECT_TABLE: u32 = 4096;
pub const SQL_SR_UPDATE_TABLE: u32 = 8192;
pub const SQL_SR_UPDATE_COLUMN: u32 = 16384;
pub const SQL_SRVC_VALUE_EXPRESSION: u32 = 1;
pub const SQL_SRVC_NULL: u32 = 2;
pub const SQL_SRVC_DEFAULT: u32 = 4;
pub const SQL_SRVC_ROW_SUBQUERY: u32 = 8;
pub const SQL_SVE_CASE: u32 = 1;
pub const SQL_SVE_CAST: u32 = 2;
pub const SQL_SVE_COALESCE: u32 = 4;
pub const SQL_SVE_NULLIF: u32 = 8;
pub const SQL_SCC_XOPEN_CLI_VERSION1: u32 = 1;
pub const SQL_SCC_ISO92_CLI: u32 = 2;
pub const SQL_US_UNION: u32 = 1;
pub const SQL_US_UNION_ALL: u32 = 2;
pub const SQL_DTC_ENLIST_EXPENSIVE: u32 = 1;
pub const SQL_DTC_UNENLIST_EXPENSIVE: u32 = 2;
pub const SQL_ASYNC_DBC_NOT_CAPABLE: u32 = 0;
pub const SQL_ASYNC_DBC_CAPABLE: u32 = 1;
pub const SQL_FETCH_FIRST_USER: u32 = 31;
pub const SQL_FETCH_FIRST_SYSTEM: u32 = 32;
pub const SQL_ENTIRE_ROWSET: u32 = 0;
pub const SQL_POSITION: u32 = 0;
pub const SQL_REFRESH: u32 = 1;
pub const SQL_UPDATE: u32 = 2;
pub const SQL_DELETE: u32 = 3;
pub const SQL_ADD: u32 = 4;
pub const SQL_SETPOS_MAX_OPTION_VALUE: u32 = 4;
pub const SQL_UPDATE_BY_BOOKMARK: u32 = 5;
pub const SQL_DELETE_BY_BOOKMARK: u32 = 6;
pub const SQL_FETCH_BY_BOOKMARK: u32 = 7;
pub const SQL_LOCK_NO_CHANGE: u32 = 0;
pub const SQL_LOCK_EXCLUSIVE: u32 = 1;
pub const SQL_LOCK_UNLOCK: u32 = 2;
pub const SQL_SETPOS_MAX_LOCK_VALUE: u32 = 2;
pub const SQL_BEST_ROWID: u32 = 1;
pub const SQL_ROWVER: u32 = 2;
pub const SQL_PC_NOT_PSEUDO: u32 = 1;
pub const SQL_QUICK: u32 = 0;
pub const SQL_ENSURE: u32 = 1;
pub const SQL_TABLE_STAT: u32 = 0;
pub const SQL_ALL_CATALOGS: &[u8; 2usize] = b"%\0";
pub const SQL_ALL_SCHEMAS: &[u8; 2usize] = b"%\0";
pub const SQL_ALL_TABLE_TYPES: &[u8; 2usize] = b"%\0";
pub const SQL_DRIVER_NOPROMPT: u32 = 0;
pub const SQL_DRIVER_COMPLETE: u32 = 1;
pub const SQL_DRIVER_PROMPT: u32 = 2;
pub const SQL_DRIVER_COMPLETE_REQUIRED: u32 = 3;
pub const SQL_FETCH_BOOKMARK: u32 = 8;
pub const SQL_ROW_SUCCESS: u32 = 0;
pub const SQL_ROW_DELETED: u32 = 1;
pub const SQL_ROW_UPDATED: u32 = 2;
pub const SQL_ROW_NOROW: u32 = 3;
pub const SQL_ROW_ADDED: u32 = 4;
pub const SQL_ROW_ERROR: u32 = 5;
pub const SQL_ROW_SUCCESS_WITH_INFO: u32 = 6;
pub const SQL_ROW_PROCEED: u32 = 0;
pub const SQL_ROW_IGNORE: u32 = 1;
pub const SQL_PARAM_SUCCESS: u32 = 0;
pub const SQL_PARAM_SUCCESS_WITH_INFO: u32 = 6;
pub const SQL_PARAM_ERROR: u32 = 5;
pub const SQL_PARAM_UNUSED: u32 = 7;
pub const SQL_PARAM_DIAG_UNAVAILABLE: u32 = 1;
pub const SQL_PARAM_PROCEED: u32 = 0;
pub const SQL_PARAM_IGNORE: u32 = 1;
pub const SQL_CASCADE: u32 = 0;
pub const SQL_RESTRICT: u32 = 1;
pub const SQL_SET_NULL: u32 = 2;
pub const SQL_NO_ACTION: u32 = 3;
pub const SQL_SET_DEFAULT: u32 = 4;
pub const SQL_INITIALLY_DEFERRED: u32 = 5;
pub const SQL_INITIALLY_IMMEDIATE: u32 = 6;
pub const SQL_NOT_DEFERRABLE: u32 = 7;
pub const SQL_PARAM_TYPE_UNKNOWN: u32 = 0;
pub const SQL_PARAM_INPUT: u32 = 1;
pub const SQL_PARAM_INPUT_OUTPUT: u32 = 2;
pub const SQL_RESULT_COL: u32 = 3;
pub const SQL_PARAM_OUTPUT: u32 = 4;
pub const SQL_RETURN_VALUE: u32 = 5;
pub const SQL_PARAM_INPUT_OUTPUT_STREAM: u32 = 8;
pub const SQL_PARAM_OUTPUT_STREAM: u32 = 16;
pub const SQL_PT_UNKNOWN: u32 = 0;
pub const SQL_PT_PROCEDURE: u32 = 1;
pub const SQL_PT_FUNCTION: u32 = 2;
pub const SQL_DATABASE_NAME: u32 = 16;
pub const SQL_CONCUR_TIMESTAMP: u32 = 3;
pub const SQL_SCROLL_FORWARD_ONLY: u32 = 0;
pub const SQL_SCROLL_KEYSET_DRIVEN: i32 = -1;
pub const SQL_SCROLL_DYNAMIC: i32 = -2;
pub const SQL_SCROLL_STATIC: i32 = -3;
pub const TRACE_VERSION: u32 = 1000;
pub const TRACE_ON: u32 = 1;
pub const TRACE_VS_EVENT_ON: u32 = 2;
pub const ODBC_VS_FLAG_UNICODE_ARG: u32 = 1;
pub const ODBC_VS_FLAG_UNICODE_COR: u32 = 2;
pub const ODBC_VS_FLAG_RETCODE: u32 = 4;
pub const ODBC_VS_FLAG_STOP: u32 = 8;
pub const SQL_API_SQLALLOCCONNECT: u32 = 1;
pub const SQL_API_SQLALLOCENV: u32 = 2;
pub const SQL_API_SQLALLOCSTMT: u32 = 3;
pub const SQL_API_SQLBINDCOL: u32 = 4;
pub const SQL_API_SQLBINDPARAM: u32 = 1002;
pub const SQL_API_SQLCANCEL: u32 = 5;
pub const SQL_API_SQLCONNECT: u32 = 7;
pub const SQL_API_SQLCOPYDESC: u32 = 1004;
pub const SQL_API_SQLDESCRIBECOL: u32 = 8;
pub const SQL_API_SQLDISCONNECT: u32 = 9;
pub const SQL_API_SQLERROR: u32 = 10;
pub const SQL_API_SQLEXECDIRECT: u32 = 11;
pub const SQL_API_SQLEXECUTE: u32 = 12;
pub const SQL_API_SQLFETCH: u32 = 13;
pub const SQL_API_SQLFREECONNECT: u32 = 14;
pub const SQL_API_SQLFREEENV: u32 = 15;
pub const SQL_API_SQLFREESTMT: u32 = 16;
pub const SQL_API_SQLGETCURSORNAME: u32 = 17;
pub const SQL_API_SQLNUMRESULTCOLS: u32 = 18;
pub const SQL_API_SQLPREPARE: u32 = 19;
pub const SQL_API_SQLROWCOUNT: u32 = 20;
pub const SQL_API_SQLSETCURSORNAME: u32 = 21;
pub const SQL_API_SQLSETDESCFIELD: u32 = 1017;
pub const SQL_API_SQLSETDESCREC: u32 = 1018;
pub const SQL_API_SQLSETENVATTR: u32 = 1019;
pub const SQL_API_SQLSETPARAM: u32 = 22;
pub const SQL_API_SQLTRANSACT: u32 = 23;
pub const SQL_API_SQLCOLUMNS: u32 = 40;
pub const SQL_API_SQLGETCONNECTOPTION: u32 = 42;
pub const SQL_API_SQLGETDATA: u32 = 43;
pub const SQL_API_SQLGETDATAINTERNAL: u32 = 174;
pub const SQL_API_SQLGETDESCFIELD: u32 = 1008;
pub const SQL_API_SQLGETDESCREC: u32 = 1009;
pub const SQL_API_SQLGETDIAGFIELD: u32 = 1010;
pub const SQL_API_SQLGETDIAGREC: u32 = 1011;
pub const SQL_API_SQLGETENVATTR: u32 = 1012;
pub const SQL_API_SQLGETFUNCTIONS: u32 = 44;
pub const SQL_API_SQLGETINFO: u32 = 45;
pub const SQL_API_SQLGETSTMTOPTION: u32 = 46;
pub const SQL_API_SQLGETTYPEINFO: u32 = 47;
pub const SQL_API_SQLPARAMDATA: u32 = 48;
pub const SQL_API_SQLPUTDATA: u32 = 49;
pub const SQL_API_SQLSETCONNECTOPTION: u32 = 50;
pub const SQL_API_SQLSETSTMTOPTION: u32 = 51;
pub const SQL_API_SQLSPECIALCOLUMNS: u32 = 52;
pub const SQL_API_SQLSTATISTICS: u32 = 53;
pub const SQL_API_SQLTABLES: u32 = 54;
pub const SQL_API_SQLDATASOURCES: u32 = 57;
pub const SQL_API_SQLSETCONNECTATTR: u32 = 1016;
pub const SQL_API_SQLSETSTMTATTR: u32 = 1020;
pub const SQL_API_SQLBINDFILETOCOL: u32 = 1250;
pub const SQL_API_SQLBINDFILETOPARAM: u32 = 1251;
pub const SQL_API_SQLSETCOLATTRIBUTES: u32 = 1252;
pub const SQL_API_SQLGETSQLCA: u32 = 1253;
pub const SQL_API_SQLSETCONNECTION: u32 = 1254;
pub const SQL_API_SQLGETDATALINKATTR: u32 = 1255;
pub const SQL_API_SQLBUILDDATALINK: u32 = 1256;
pub const SQL_API_SQLNEXTRESULT: u32 = 1257;
pub const SQL_API_SQLCREATEDB: u32 = 1258;
pub const SQL_API_SQLDROPDB: u32 = 1259;
pub const SQL_API_SQLCREATEPKG: u32 = 1260;
pub const SQL_API_SQLDROPPKG: u32 = 1261;
pub const SQL_API_SQLEXTENDEDPREPARE: u32 = 1296;
pub const SQL_API_SQLEXTENDEDBIND: u32 = 1297;
pub const SQL_API_SQLEXTENDEDDESCRIBE: u32 = 1298;
pub const SQL_API_SQLRELOADCONFIG: u32 = 1299;
pub const SQL_API_SQLFETCHSCROLL: u32 = 1021;
pub const SQL_API_SQLGETLENGTH: u32 = 1022;
pub const SQL_API_SQLGETPOSITION: u32 = 1023;
pub const SQL_API_SQLGETSUBSTRING: u32 = 1024;
pub const SQL_API_SQLEXTENDEDPROCEDURES: u32 = 1025;
pub const SQL_API_SQLEXTENDEDPROCEDURECOLUMNS: u32 = 1026;
pub const SQL_API_SQLALLOCHANDLE: u32 = 1001;
pub const SQL_API_SQLFREEHANDLE: u32 = 1006;
pub const SQL_API_SQLCLOSECURSOR: u32 = 1003;
pub const SQL_API_SQLENDTRAN: u32 = 1005;
pub const SQL_API_SQLCOLATTRIBUTE: u32 = 6;
pub const SQL_API_SQLGETSTMTATTR: u32 = 1014;
pub const SQL_API_SQLGETCONNECTATTR: u32 = 1007;
pub const SQL_EXT_API_LAST: u32 = 72;
pub const SQL_MAX_DRIVER_CONNECTIONS: u32 = 0;
pub const SQL_MAXIMUM_DRIVER_CONNECTIONS: u32 = 0;
pub const SQL_MAX_CONCURRENT_ACTIVITIES: u32 = 1;
pub const SQL_MAXIMUM_CONCURRENT_ACTIVITIES: u32 = 1;
pub const SQL_DROP_MODULE: u32 = 2600;
pub const SQL_MODULE_USAGE: u32 = 2601;
pub const SQL_CREATE_MODULE: u32 = 2602;
pub const SQL_MAX_MODULE_NAME_LEN: u32 = 2603;
pub const SQL_DRIVER_BLDLEVEL: u32 = 2604;
pub const SQL_DATALINK_URL: &[u8; 4usize] = b"URL\0";
pub const SQL_ATTR_DATALINK_COMMENT: u32 = 1;
pub const SQL_ATTR_DATALINK_LINKTYPE: u32 = 2;
pub const SQL_ATTR_DATALINK_URLCOMPLETE: u32 = 3;
pub const SQL_ATTR_DATALINK_URLPATH: u32 = 4;
pub const SQL_ATTR_DATALINK_URLPATHONLY: u32 = 5;
pub const SQL_ATTR_DATALINK_URLSCHEME: u32 = 6;
pub const SQL_ATTR_DATALINK_URLSERVER: u32 = 7;
pub const SQL_DATA_SOURCE_NAME: u32 = 2;
pub const SQL_FETCH_DIRECTION: u32 = 8;
pub const SQL_SERVER_NAME: u32 = 13;
pub const SQL_SEARCH_PATTERN_ESCAPE: u32 = 14;
pub const SQL_DBMS_NAME: u32 = 17;
pub const SQL_DBMS_VER: u32 = 18;
pub const SQL_ACCESSIBLE_TABLES: u32 = 19;
pub const SQL_ACCESSIBLE_PROCEDURES: u32 = 20;
pub const SQL_CURSOR_COMMIT_BEHAVIOR: u32 = 23;
pub const SQL_DATA_SOURCE_READ_ONLY: u32 = 25;
pub const SQL_DEFAULT_TXN_ISOLATION: u32 = 26;
pub const SQL_IDENTIFIER_CASE: u32 = 28;
pub const SQL_IDENTIFIER_QUOTE_CHAR: u32 = 29;
pub const SQL_MAX_COLUMN_NAME_LEN: u32 = 30;
pub const SQL_MAXIMUM_COLUMN_NAME_LENGTH: u32 = 30;
pub const SQL_MAX_CURSOR_NAME_LEN: u32 = 31;
pub const SQL_MAXIMUM_CURSOR_NAME_LENGTH: u32 = 31;
pub const SQL_MAX_TABLE_NAME_LEN: u32 = 35;
pub const SQL_SCROLL_CONCURRENCY: u32 = 43;
pub const SQL_TXN_CAPABLE: u32 = 46;
pub const SQL_TRANSACTION_CAPABLE: u32 = 46;
pub const SQL_USER_NAME: u32 = 47;
pub const SQL_TXN_ISOLATION_OPTION: u32 = 72;
pub const SQL_TRANSACTION_ISOLATION_OPTION: u32 = 72;
pub const SQL_GETDATA_EXTENSIONS: u32 = 81;
pub const SQL_NULL_COLLATION: u32 = 85;
pub const SQL_ALTER_TABLE: u32 = 86;
pub const SQL_ORDER_BY_COLUMNS_IN_SELECT: u32 = 90;
pub const SQL_SPECIAL_CHARACTERS: u32 = 94;
pub const SQL_MAX_COLUMNS_IN_GROUP_BY: u32 = 97;
pub const SQL_MAXIMUM_COLUMNS_IN_GROUP_BY: u32 = 97;
pub const SQL_MAX_COLUMNS_IN_INDEX: u32 = 98;
pub const SQL_MAXIMUM_COLUMNS_IN_INDEX: u32 = 98;
pub const SQL_MAX_COLUMNS_IN_ORDER_BY: u32 = 99;
pub const SQL_MAXIMUM_COLUMNS_IN_ORDER_BY: u32 = 99;
pub const SQL_MAX_COLUMNS_IN_SELECT: u32 = 100;
pub const SQL_MAXIMUM_COLUMNS_IN_SELECT: u32 = 100;
pub const SQL_MAX_COLUMNS_IN_TABLE: u32 = 101;
pub const SQL_MAX_INDEX_SIZE: u32 = 102;
pub const SQL_MAXIMUM_INDEX_SIZE: u32 = 102;
pub const SQL_MAX_ROW_SIZE: u32 = 104;
pub const SQL_MAXIMUM_ROW_SIZE: u32 = 104;
pub const SQL_MAX_STATEMENT_LEN: u32 = 105;
pub const SQL_MAXIMUM_STATEMENT_LENGTH: u32 = 105;
pub const SQL_MAX_TABLES_IN_SELECT: u32 = 106;
pub const SQL_MAXIMUM_TABLES_IN_SELECT: u32 = 106;
pub const SQL_MAX_USER_NAME_LEN: u32 = 107;
pub const SQL_MAXIMUM_USER_NAME_LENGTH: u32 = 107;
pub const SQL_MAX_SCHEMA_NAME_LEN: u32 = 32;
pub const SQL_MAXIMUM_SCHEMA_NAME_LENGTH: u32 = 32;
pub const SQL_MAX_CATALOG_NAME_LEN: u32 = 34;
pub const SQL_MAXIMUM_CATALOG_NAME_LENGTH: u32 = 34;
pub const SQL_OJ_CAPABILITIES: u32 = 115;
pub const SQL_CONFIG_KEYWORDS: u32 = 174;
pub const SQL_OUTER_JOIN_CAPABILITIES: u32 = 115;
pub const SQL_XOPEN_CLI_YEAR: u32 = 10000;
pub const SQL_CURSOR_SENSITIVITY: u32 = 10001;
pub const SQL_DESCRIBE_PARAMETER: u32 = 10002;
pub const SQL_CATALOG_NAME: u32 = 10003;
pub const SQL_COLLATION_SEQ: u32 = 10004;
pub const SQL_MAX_IDENTIFIER_LEN: u32 = 10005;
pub const SQL_MAXIMUM_IDENTIFIER_LENGTH: u32 = 10005;
pub const SQL_INTEGRITY: u32 = 73;
pub const SQL_DATABASE_CODEPAGE: u32 = 2519;
pub const SQL_APPLICATION_CODEPAGE: u32 = 2520;
pub const SQL_CONNECT_CODEPAGE: u32 = 2521;
pub const SQL_ATTR_DB2_APPLICATION_ID: u32 = 2532;
pub const SQL_ATTR_DB2_APPLICATION_HANDLE: u32 = 2533;
pub const SQL_ATTR_HANDLE_XA_ASSOCIATED: u32 = 2535;
pub const SQL_DB2_DRIVER_VER: u32 = 2550;
pub const SQL_ATTR_XML_DECLARATION: u32 = 2552;
pub const SQL_ATTR_CURRENT_IMPLICIT_XMLPARSE_OPTION: u32 = 2553;
pub const SQL_ATTR_XQUERY_STATEMENT: u32 = 2557;
pub const SQL_DB2_DRIVER_TYPE: u32 = 2567;
pub const SQL_INPUT_CHAR_CONVFACTOR: u32 = 2581;
pub const SQL_OUTPUT_CHAR_CONVFACTOR: u32 = 2582;
pub const SQL_ATTR_REPLACE_QUOTED_LITERALS: u32 = 2586;
pub const SQL_ATTR_REPORT_TIMESTAMP_TRUNC_AS_WARN: u32 = 2587;
pub const SQL_ATTR_CLIENT_ENCALG: u32 = 2589;
pub const SQL_ATTR_CONCURRENT_ACCESS_RESOLUTION: u32 = 2595;
pub const SQL_ATTR_REPORT_SEAMLESSFAILOVER_WARNING: u32 = 2605;
pub const SQL_CONCURRENT_ACCESS_RESOLUTION_UNSET: u32 = 0;
pub const SQL_USE_CURRENTLY_COMMITTED: u32 = 1;
pub const SQL_WAIT_FOR_OUTCOME: u32 = 2;
pub const SQL_SKIP_LOCKED_DATA: u32 = 3;
pub const SQL_DBMS_FUNCTIONLVL: u32 = 203;
pub const SQL_CLI_STMT_UNDEFINED: u32 = 0;
pub const SQL_CLI_STMT_ALTER_TABLE: u32 = 1;
pub const SQL_CLI_STMT_CREATE_INDEX: u32 = 5;
pub const SQL_CLI_STMT_CREATE_TABLE: u32 = 6;
pub const SQL_CLI_STMT_CREATE_VIEW: u32 = 7;
pub const SQL_CLI_STMT_DELETE_SEARCHED: u32 = 8;
pub const SQL_CLI_STMT_DELETE_POSITIONED: u32 = 9;
pub const SQL_CLI_STMT_DROP_PACKAGE: u32 = 10;
pub const SQL_CLI_STMT_DROP_INDEX: u32 = 11;
pub const SQL_CLI_STMT_DROP_TABLE: u32 = 12;
pub const SQL_CLI_STMT_DROP_VIEW: u32 = 13;
pub const SQL_CLI_STMT_GRANT: u32 = 14;
pub const SQL_CLI_STMT_INSERT: u32 = 15;
pub const SQL_CLI_STMT_REVOKE: u32 = 16;
pub const SQL_CLI_STMT_SELECT: u32 = 18;
pub const SQL_CLI_STMT_UPDATE_SEARCHED: u32 = 19;
pub const SQL_CLI_STMT_UPDATE_POSITIONED: u32 = 20;
pub const SQL_CLI_STMT_CALL: u32 = 24;
pub const SQL_CLI_STMT_SELECT_FOR_UPDATE: u32 = 29;
pub const SQL_CLI_STMT_WITH: u32 = 30;
pub const SQL_CLI_STMT_SELECT_FOR_FETCH: u32 = 31;
pub const SQL_CLI_STMT_VALUES: u32 = 32;
pub const SQL_CLI_STMT_CREATE_TRIGGER: u32 = 34;
pub const SQL_CLI_STMT_SELECT_OPTIMIZE_FOR_NROWS: u32 = 39;
pub const SQL_CLI_STMT_SELECT_INTO: u32 = 40;
pub const SQL_CLI_STMT_CREATE_PROCEDURE: u32 = 41;
pub const SQL_CLI_STMT_CREATE_FUNCTION: u32 = 42;
pub const SQL_CLI_STMT_INSERT_VALUES: u32 = 45;
pub const SQL_CLI_STMT_SET_CURRENT_QUERY_OPT: u32 = 46;
pub const SQL_CLI_STMT_MERGE: u32 = 56;
pub const SQL_CLI_STMT_XQUERY: u32 = 59;
pub const SQL_CLI_STMT_SET: u32 = 62;
pub const SQL_CLI_STMT_ALTER_PROCEDURE: u32 = 63;
pub const SQL_CLI_STMT_CLOSE_DATABASE: u32 = 64;
pub const SQL_CLI_STMT_CREATE_DATABASE: u32 = 65;
pub const SQL_CLI_STMT_DROP_DATABASE: u32 = 66;
pub const SQL_CLI_STMT_ANONYMOUS_BLOCK: u32 = 72;
pub const SQL_IBM_ALTERTABLEVARCHAR: u32 = 1000;
pub const SQL_AT_ADD_COLUMN: u32 = 1;
pub const SQL_AT_DROP_COLUMN: u32 = 2;
pub const SQL_AT_ADD_CONSTRAINT: u32 = 8;
pub const SQL_CB_DELETE: u32 = 0;
pub const SQL_CB_CLOSE: u32 = 1;
pub const SQL_CB_PRESERVE: u32 = 2;
pub const SQL_IC_UPPER: u32 = 1;
pub const SQL_IC_LOWER: u32 = 2;
pub const SQL_IC_SENSITIVE: u32 = 3;
pub const SQL_IC_MIXED: u32 = 4;
pub const SQL_TC_NONE: u32 = 0;
pub const SQL_TC_DML: u32 = 1;
pub const SQL_TC_ALL: u32 = 2;
pub const SQL_TC_DDL_COMMIT: u32 = 3;
pub const SQL_TC_DDL_IGNORE: u32 = 4;
pub const SQL_SCCO_READ_ONLY: u32 = 1;
pub const SQL_SCCO_LOCK: u32 = 2;
pub const SQL_SCCO_OPT_ROWVER: u32 = 4;
pub const SQL_SCCO_OPT_VALUES: u32 = 8;
pub const SQL_FD_FETCH_NEXT: u32 = 1;
pub const SQL_FD_FETCH_FIRST: u32 = 2;
pub const SQL_FD_FETCH_LAST: u32 = 4;
pub const SQL_FD_FETCH_PRIOR: u32 = 8;
pub const SQL_FD_FETCH_ABSOLUTE: u32 = 16;
pub const SQL_FD_FETCH_RELATIVE: u32 = 32;
pub const SQL_FD_FETCH_RESUME: u32 = 64;
pub const SQL_TXN_READ_UNCOMMITTED: u32 = 1;
pub const SQL_TRANSACTION_READ_UNCOMMITTED: u32 = 1;
pub const SQL_TXN_READ_COMMITTED: u32 = 2;
pub const SQL_TRANSACTION_READ_COMMITTED: u32 = 2;
pub const SQL_TXN_REPEATABLE_READ: u32 = 4;
pub const SQL_TRANSACTION_REPEATABLE_READ: u32 = 4;
pub const SQL_TXN_SERIALIZABLE: u32 = 8;
pub const SQL_TRANSACTION_SERIALIZABLE: u32 = 8;
pub const SQL_TXN_NOCOMMIT: u32 = 32;
pub const SQL_TRANSACTION_NOCOMMIT: u32 = 32;
pub const SQL_TXN_IDS_CURSOR_STABILITY: u32 = 64;
pub const SQL_TRANSACTION_IDS_CURSOR_STABILITY: u32 = 64;
pub const SQL_TXN_IDS_LAST_COMMITTED: u32 = 128;
pub const SQL_TRANSACTION_IDS_LAST_COMMITTED: u32 = 128;
pub const SQL_GD_ANY_COLUMN: u32 = 1;
pub const SQL_GD_ANY_ORDER: u32 = 2;
pub const SQL_OJ_LEFT: u32 = 1;
pub const SQL_OJ_RIGHT: u32 = 2;
pub const SQL_OJ_FULL: u32 = 4;
pub const SQL_OJ_NESTED: u32 = 8;
pub const SQL_OJ_NOT_ORDERED: u32 = 16;
pub const SQL_OJ_INNER: u32 = 32;
pub const SQL_OJ_ALL_COMPARISON_OPS: u32 = 64;
pub const SQL_CLI_DRIVER_TYPE_UNDEFINED: u32 = 0;
pub const SQL_CLI_DRIVER_RUNTIME_CLIENT: u32 = 1;
pub const SQL_CLI_DRIVER_CLI_DRIVER: u32 = 2;
pub const SQL_ALL_TYPES: u32 = 0;
pub const SQL_ATTR_AUTO_IPD: u32 = 10001;
pub const SQL_ATTR_APP_ROW_DESC: u32 = 10010;
pub const SQL_ATTR_APP_PARAM_DESC: u32 = 10011;
pub const SQL_ATTR_IMP_ROW_DESC: u32 = 10012;
pub const SQL_ATTR_IMP_PARAM_DESC: u32 = 10013;
pub const SQL_ATTR_METADATA_ID: u32 = 10014;
pub const SQL_ATTR_CURSOR_SCROLLABLE: i32 = -1;
pub const SQL_ATTR_CURSOR_SENSITIVITY: i32 = -2;
pub const SQL_NONSCROLLABLE: u32 = 0;
pub const SQL_SCROLLABLE: u32 = 1;
pub const SQL_CURSOR_HOLD: u32 = 1250;
pub const SQL_ATTR_CURSOR_HOLD: u32 = 1250;
pub const SQL_NODESCRIBE_OUTPUT: u32 = 1251;
pub const SQL_ATTR_NODESCRIBE_OUTPUT: u32 = 1251;
pub const SQL_NODESCRIBE_INPUT: u32 = 1264;
pub const SQL_ATTR_NODESCRIBE_INPUT: u32 = 1264;
pub const SQL_NODESCRIBE: u32 = 1251;
pub const SQL_ATTR_NODESCRIBE: u32 = 1251;
pub const SQL_CLOSE_BEHAVIOR: u32 = 1257;
pub const SQL_ATTR_CLOSE_BEHAVIOR: u32 = 1257;
pub const SQL_ATTR_CLOSEOPEN: u32 = 1265;
pub const SQL_ATTR_CURRENT_PACKAGE_SET: u32 = 1276;
pub const SQL_ATTR_DEFERRED_PREPARE: u32 = 1277;
pub const SQL_ATTR_EARLYCLOSE: u32 = 1268;
pub const SQL_ATTR_PROCESSCTL: u32 = 1278;
pub const SQL_ATTR_PREFETCH: u32 = 1285;
pub const SQL_ATTR_ENABLE_IPD_SETTING: u32 = 1286;
pub const SQL_ATTR_RETRYONERROR: u32 = 121;
pub const SQL_DESC_DESCRIPTOR_TYPE: u32 = 1287;
pub const SQL_ATTR_OPTIMIZE_SQLCOLUMNS: u32 = 1288;
pub const SQL_ATTR_MEM_DEBUG_DUMP: u32 = 1289;
pub const SQL_ATTR_CONNECT_NODE: u32 = 1290;
pub const SQL_ATTR_CONNECT_WITH_XA: u32 = 1291;
pub const SQL_ATTR_GET_XA_RESOURCE: u32 = 1292;
pub const SQL_ATTR_DB2_SQLERRP: u32 = 2451;
pub const SQL_ATTR_SERVER_MSGTXT_SP: u32 = 2452;
pub const SQL_ATTR_OPTIMIZE_FOR_NROWS: u32 = 2450;
pub const SQL_ATTR_QUERY_OPTIMIZATION_LEVEL: u32 = 1293;
pub const SQL_ATTR_USE_LIGHT_OUTPUT_SQLDA: u32 = 1298;
pub const SQL_ATTR_CURSOR_BLOCK_NUM_ROWS: u32 = 2453;
pub const SQL_ATTR_CURSOR_BLOCK_EARLY_CLOSE: u32 = 2454;
pub const SQL_ATTR_SERVER_MSGTXT_MASK: u32 = 2455;
pub const SQL_ATTR_USE_LIGHT_INPUT_SQLDA: u32 = 2458;
pub const SQL_ATTR_BLOCK_FOR_NROWS: u32 = 2459;
pub const SQL_ATTR_OPTIMIZE_ROWS_FOR_BLOCKING: u32 = 2460;
pub const SQL_ATTR_STATICMODE: u32 = 2467;
pub const SQL_ATTR_DB2_MESSAGE_PREFIX: u32 = 2468;
pub const SQL_ATTR_CALL_RETVAL_AS_PARM: u32 = 2469;
pub const SQL_ATTR_CALL_RETURN: u32 = 2470;
pub const SQL_ATTR_RETURN_USER_DEFINED_TYPES: u32 = 2471;
pub const SQL_ATTR_ENABLE_EXTENDED_PARAMDATA: u32 = 2472;
pub const SQL_ATTR_APP_TYPE: u32 = 2473;
pub const SQL_ATTR_TRANSFORM_GROUP: u32 = 2474;
pub const SQL_ATTR_DESCRIBE_CALL: u32 = 2476;
pub const SQL_ATTR_AUTOCOMMCLEANUP: u32 = 2477;
pub const SQL_ATTR_USEMALLOC: u32 = 2478;
pub const SQL_ATTR_PRESERVE_LOCALE: u32 = 2479;
pub const SQL_ATTR_MAPGRAPHIC: u32 = 2480;
pub const SQL_ATTR_INSERT_BUFFERING: u32 = 2481;
pub const SQL_ATTR_USE_LOAD_API: u32 = 2482;
pub const SQL_ATTR_LOAD_RECOVERABLE: u32 = 2483;
pub const SQL_ATTR_LOAD_COPY_LOCATION: u32 = 2484;
pub const SQL_ATTR_LOAD_MESSAGE_FILE: u32 = 2485;
pub const SQL_ATTR_LOAD_SAVECOUNT: u32 = 2486;
pub const SQL_ATTR_LOAD_CPU_PARALLELISM: u32 = 2487;
pub const SQL_ATTR_LOAD_DISK_PARALLELISM: u32 = 2488;
pub const SQL_ATTR_LOAD_INDEXING_MODE: u32 = 2489;
pub const SQL_ATTR_LOAD_STATS_MODE: u32 = 2490;
pub const SQL_ATTR_LOAD_TEMP_FILES_PATH: u32 = 2491;
pub const SQL_ATTR_LOAD_DATA_BUFFER_SIZE: u32 = 2492;
pub const SQL_ATTR_LOAD_MODIFIED_BY: u32 = 2493;
pub const SQL_ATTR_DB2_RESERVED_2494: u32 = 2494;
pub const SQL_ATTR_DESCRIBE_BEHAVIOR: u32 = 2495;
pub const SQL_ATTR_FETCH_SENSITIVITY: u32 = 2496;
pub const SQL_ATTR_DB2_RESERVED_2497: u32 = 2497;
pub const SQL_ATTR_CLIENT_LOB_BUFFERING: u32 = 2498;
pub const SQL_ATTR_SKIP_TRACE: u32 = 2499;
pub const SQL_ATTR_LOAD_INFO: u32 = 2501;
pub const SQL_ATTR_DESCRIBE_INPUT_ON_PREPARE: u32 = 2505;
pub const SQL_ATTR_DESCRIBE_OUTPUT_LEVEL: u32 = 2506;
pub const SQL_ATTR_CURRENT_PACKAGE_PATH: u32 = 2509;
pub const SQL_ATTR_INFO_PROGRAMID: u32 = 2511;
pub const SQL_ATTR_INFO_PROGRAMNAME: u32 = 2516;
pub const SQL_ATTR_FREE_LOCATORS_ON_FETCH: u32 = 2518;
pub const SQL_ATTR_KEEP_DYNAMIC: u32 = 2522;
pub const SQL_ATTR_LOAD_ROWS_READ_PTR: u32 = 2524;
pub const SQL_ATTR_LOAD_ROWS_SKIPPED_PTR: u32 = 2525;
pub const SQL_ATTR_LOAD_ROWS_COMMITTED_PTR: u32 = 2526;
pub const SQL_ATTR_LOAD_ROWS_LOADED_PTR: u32 = 2527;
pub const SQL_ATTR_LOAD_ROWS_REJECTED_PTR: u32 = 2528;
pub const SQL_ATTR_LOAD_ROWS_DELETED_PTR: u32 = 2529;
pub const SQL_ATTR_LOAD_INFO_VER: u32 = 2530;
pub const SQL_ATTR_SET_SSA: u32 = 2531;
pub const SQL_ATTR_BLOCK_LOBS: u32 = 2534;
pub const SQL_ATTR_LOAD_ACCESS_LEVEL: u32 = 2536;
pub const SQL_ATTR_MAPCHAR: u32 = 2546;
pub const SQL_ATTR_ARM_CORRELATOR: u32 = 2554;
pub const SQL_ATTR_CLIENT_DEBUGINFO: u32 = 2556;
pub const SQL_ATTR_GET_GENERATED_VALUE: u32 = 2583;
pub const SQL_ATTR_GET_SERIAL_VALUE: u32 = 2584;
pub const SQL_ATTR_INTERLEAVED_PUTDATA: u32 = 2591;
pub const SQL_ATTR_FORCE_ROLLBACK: u32 = 2596;
pub const SQL_ATTR_STMT_CONCENTRATOR: u32 = 2597;
pub const SQL_ATTR_LOAD_REPLACE_OPTION: u32 = 3036;
pub const SQL_ATTR_SESSION_GLOBAL_VAR: u32 = 3044;
pub const SQL_ATTR_SPECIAL_REGISTER: u32 = 3049;
pub const SQL_STMT_CONCENTRATOR_OFF: u32 = 1;
pub const SQL_STMT_CONCENTRATOR_WITH_LITERALS: u32 = 2;
pub const SQL_INFO_LAST: u32 = 174;
pub const SQL_INFO_DRIVER_START: u32 = 1000;
pub const SQL_FORCE_ROLLBACK_ON: u32 = 1;
pub const SQL_FORCE_ROLLBACK_OFF: u32 = 0;
pub const SQL_FORCE_ROLLBACK_DEFAULT: u32 = 0;
pub const SQL_DESCRIBE_NONE: u32 = 0;
pub const SQL_DESCRIBE_LIGHT: u32 = 1;
pub const SQL_DESCRIBE_REGULAR: u32 = 2;
pub const SQL_DESCRIBE_EXTENDED: u32 = 3;
pub const SQL_USE_LOAD_OFF: u32 = 0;
pub const SQL_USE_LOAD_INSERT: u32 = 1;
pub const SQL_USE_LOAD_REPLACE: u32 = 2;
pub const SQL_USE_LOAD_RESTART: u32 = 3;
pub const SQL_USE_LOAD_TERMINATE: u32 = 4;
pub const SQL_USE_LOAD_WITH_ET: u32 = 5;
pub const SQL_LOAD_REPLACE_DEFAULT: u32 = 0;
pub const SQL_LOAD_KEEPDICTIONARY: u32 = 1;
pub const SQL_LOAD_RESETDICTIONARY: u32 = 2;
pub const SQL_LOAD_RESETDICTIONARYONLY: u32 = 3;
pub const SQL_PREFETCH_ON: u32 = 1;
pub const SQL_PREFETCH_OFF: u32 = 0;
pub const SQL_PREFETCH_DEFAULT: u32 = 0;
pub const SQL_CC_NO_RELEASE: u32 = 0;
pub const SQL_CC_RELEASE: u32 = 1;
pub const SQL_CC_DEFAULT: u32 = 0;
pub const SQL_RETRYONERROR_OFF: u32 = 0;
pub const SQL_RETRYONERROR_ON: u32 = 1;
pub const SQL_RETRYONERROR_DEFAULT: u32 = 1;
pub const SQL_RETRYBINDONERROR_OFF: u32 = 0;
pub const SQL_RETRYBINDONERROR_ON: u32 = 1;
pub const SQL_RETRYBINDONERROR_DEFAULT: u32 = 1;
pub const SQL_ALLOW_INTERLEAVED_GETDATA_OFF: u32 = 0;
pub const SQL_ALLOW_INTERLEAVED_GETDATA_ON: u32 = 1;
pub const SQL_ALLOW_INTERLEAVED_GETDATA_DEFAULT: u32 = 0;
pub const SQL_INTERLEAVED_STREAM_PUTDATA_OFF: u32 = 0;
pub const SQL_INTERLEAVED_STREAM_PUTDATA_ON: u32 = 1;
pub const SQL_OVERRIDE_CODEPAGE_ON: u32 = 1;
pub const SQL_OVERRIDE_CODEPAGE_OFF: u32 = 0;
pub const SQL_DEFERRED_PREPARE_ON: u32 = 1;
pub const SQL_DEFERRED_PREPARE_OFF: u32 = 0;
pub const SQL_DEFERRED_PREPARE_DEFAULT: u32 = 1;
pub const SQL_EARLYCLOSE_ON: u32 = 1;
pub const SQL_EARLYCLOSE_OFF: u32 = 0;
pub const SQL_EARLYCLOSE_SERVER: u32 = 2;
pub const SQL_EARLYCLOSE_DEFAULT: u32 = 1;
pub const SQL_APP_TYPE_ODBC: u32 = 1;
pub const SQL_APP_TYPE_OLEDB: u32 = 2;
pub const SQL_APP_TYPE_JDBC: u32 = 3;
pub const SQL_APP_TYPE_ADONET: u32 = 4;
pub const SQL_APP_TYPE_DRDAWRAPPER: u32 = 5;
pub const SQL_APP_TYPE_OCI: u32 = 6;
pub const SQL_APP_TYPE_DEFAULT: u32 = 1;
pub const SQL_PROCESSCTL_NOTHREAD: u32 = 1;
pub const SQL_PROCESSCTL_NOFORK: u32 = 2;
pub const SQL_PROCESSCTL_SHARESTMTDESC: u32 = 4;
pub const SQL_PROCESSCTL_MULTICONNECT3: u32 = 8;
pub const SQL_FALSE: u32 = 0;
pub const SQL_TRUE: u32 = 1;
pub const SQL_CURSOR_HOLD_ON: u32 = 1;
pub const SQL_CURSOR_HOLD_OFF: u32 = 0;
pub const SQL_CURSOR_HOLD_DEFAULT: u32 = 1;
pub const SQL_NODESCRIBE_ON: u32 = 1;
pub const SQL_NODESCRIBE_OFF: u32 = 0;
pub const SQL_NODESCRIBE_DEFAULT: u32 = 0;
pub const SQL_DESCRIBE_CALL_NEVER: u32 = 0;
pub const SQL_DESCRIBE_CALL_BEFORE: u32 = 1;
pub const SQL_DESCRIBE_CALL_ON_ERROR: u32 = 2;
pub const SQL_DESCRIBE_CALL_DEFAULT: i32 = -1;
pub const SQL_CLIENTLOB_USE_LOCATORS: u32 = 0;
pub const SQL_CLIENTLOB_BUFFER_UNBOUND_LOBS: u32 = 1;
pub const SQL_CLIENTLOB_DEFAULT: u32 = 0;
pub const SQL_CLIENT_ENCALG_NOT_SET: u32 = 0;
pub const SQL_CLIENT_ENCALG_ANY: u32 = 1;
pub const SQL_CLIENT_ENCALG_AES_ONLY: u32 = 2;
pub const SQL_COMMITONEOF_OFF: u32 = 0;
pub const SQL_COMMITONEOF_ON: u32 = 1;
pub const SQL_WCHARTYPE: u32 = 1252;
pub const SQL_LONGDATA_COMPAT: u32 = 1253;
pub const SQL_CURRENT_SCHEMA: u32 = 1254;
pub const SQL_DB2EXPLAIN: u32 = 1258;
pub const SQL_DB2ESTIMATE: u32 = 1259;
pub const SQL_PARAMOPT_ATOMIC: u32 = 1260;
pub const SQL_STMTTXN_ISOLATION: u32 = 1261;
pub const SQL_MAXCONN: u32 = 1262;
pub const SQL_ATTR_CLISCHEMA: u32 = 1280;
pub const SQL_ATTR_INFO_USERID: u32 = 1281;
pub const SQL_ATTR_INFO_WRKSTNNAME: u32 = 1282;
pub const SQL_ATTR_INFO_APPLNAME: u32 = 1283;
pub const SQL_ATTR_INFO_ACCTSTR: u32 = 1284;
pub const SQL_ATTR_AUTOCOMMIT_NOCOMMIT: u32 = 2462;
pub const SQL_ATTR_QUERY_PATROLLER: u32 = 2466;
pub const SQL_ATTR_CHAINING_BEGIN: u32 = 2464;
pub const SQL_ATTR_CHAINING_END: u32 = 2465;
pub const SQL_ATTR_EXTENDEDBIND: u32 = 2475;
pub const SQL_ATTR_GRAPHIC_UNICODESERVER: u32 = 2503;
pub const SQL_ATTR_RETURN_CHAR_AS_WCHAR_OLEDB: u32 = 2517;
pub const SQL_ATTR_GATEWAY_CONNECTED: u32 = 2537;
pub const SQL_ATTR_SQLCOLUMNS_SORT_BY_ORDINAL_OLEDB: u32 = 2542;
pub const SQL_ATTR_REPORT_ISLONG_FOR_LONGTYPES_OLEDB: u32 = 2543;
pub const SQL_ATTR_PING_DB: u32 = 2545;
pub const SQL_ATTR_RECEIVE_TIMEOUT: u32 = 2547;
pub const SQL_ATTR_REOPT: u32 = 2548;
pub const SQL_ATTR_LOB_CACHE_SIZE: u32 = 2555;
pub const SQL_ATTR_STREAM_GETDATA: u32 = 2558;
pub const SQL_ATTR_APP_USES_LOB_LOCATOR: u32 = 2559;
pub const SQL_ATTR_MAX_LOB_BLOCK_SIZE: u32 = 2560;
pub const SQL_ATTR_USE_TRUSTED_CONTEXT: u32 = 2561;
pub const SQL_ATTR_TRUSTED_CONTEXT_USERID: u32 = 2562;
pub const SQL_ATTR_TRUSTED_CONTEXT_PASSWORD: u32 = 2563;
pub const SQL_ATTR_USER_REGISTRY_NAME: u32 = 2564;
pub const SQL_ATTR_DECFLOAT_ROUNDING_MODE: u32 = 2565;
pub const SQL_ATTR_APPEND_FOR_FETCH_ONLY: u32 = 2573;
pub const SQL_ATTR_ONLY_USE_BIG_PACKAGES: u32 = 2577;
pub const SQL_ATTR_NONATMOIC_BUFFER_INSERT: u32 = 2588;
pub const SQL_ATTR_ROWCOUNT_PREFETCH: u32 = 2592;
pub const SQL_ATTR_PING_REQUEST_PACKET_SIZE: u32 = 2593;
pub const SQL_ATTR_PING_NTIMES: u32 = 2594;
pub const SQL_ATTR_ALLOW_INTERLEAVED_GETDATA: u32 = 2599;
pub const SQL_ATTR_INTERLEAVED_STREAM_PUTDATA: u32 = 3000;
pub const SQL_ATTR_FET_BUF_SIZE: u32 = 3001;
pub const SQL_ATTR_CLIENT_CODEPAGE: u32 = 3002;
pub const SQL_ATTR_EXTENDED_INDICATORS: u32 = 3003;
pub const SQL_ATTR_SESSION_TIME_ZONE: u32 = 3004;
pub const SQL_ATTR_CLIENT_TIME_ZONE: u32 = 3005;
pub const SQL_ATTR_NETWORK_STATISTICS: u32 = 3006;
pub const SQL_ATTR_OVERRIDE_CHARACTER_CODEPAGE: u32 = 3007;
pub const SQL_ATTR_GET_LATEST_MEMBER: u32 = 3008;
pub const SQL_ATTR_CO_CAPTUREONPREPARE: u32 = 3009;
pub const SQL_ATTR_RETRYBINDONERROR: u32 = 3010;
pub const SQL_ATTR_COMMITONEOF: u32 = 3011;
pub const SQL_ATTR_PARC_BATCH: u32 = 3012;
pub const SQL_ATTR_COLUMNWISE_MRI: u32 = 3013;
pub const SQL_ATTR_OVERRIDE_CODEPAGE: u32 = 3014;
pub const SQL_ATTR_SQLCODEMAP: u32 = 3015;
pub const SQL_ATTR_ISREADONLYSQL: u32 = 3016;
pub const SQL_ATTR_DBC_SYS_NAMING: u32 = 3017;
pub const SQL_ATTR_FREE_MEMORY_ON_STMTCLOSE: u32 = 3018;
pub const SQL_ATTR_OVERRIDE_PRIMARY_AFFINITY: u32 = 3020;
pub const SQL_ATTR_STREAM_OUTPUTLOB_ON_CALL: u32 = 3021;
pub const SQL_ATTR_CACHE_USRLIBL: u32 = 3022;
pub const SQL_ATTR_GET_LATEST_MEMBER_NAME: u32 = 3023;
pub const SQL_ATTR_INFO_CRRTKN: u32 = 3024;
pub const SQL_ATTR_DATE_FMT: u32 = 3025;
pub const SQL_ATTR_DATE_SEP: u32 = 3026;
pub const SQL_ATTR_TIME_FMT: u32 = 3027;
pub const SQL_ATTR_TIME_SEP: u32 = 3028;
pub const SQL_ATTR_DECIMAL_SEP: u32 = 3029;
pub const SQL_ATTR_READ_ONLY_CONNECTION: u32 = 3030;
pub const SQL_ATTR_CONFIG_KEYWORDS_ARRAY_SIZE: u32 = 3031;
pub const SQL_ATTR_CONFIG_KEYWORDS_MAXLEN: u32 = 3032;
pub const SQL_ATTR_RETRY_ON_MERGE: u32 = 3033;
pub const SQL_ATTR_DETECT_READ_ONLY_TXN: u32 = 3034;
pub const SQL_ATTR_IGNORE_SERVER_LIST: u32 = 3035;
pub const SQL_ATTR_DB2ZLOAD_LOADSTMT: u32 = 3037;
pub const SQL_ATTR_DB2ZLOAD_RECDELIM: u32 = 3038;
pub const SQL_ATTR_DB2ZLOAD_BEGIN: u32 = 3039;
pub const SQL_ATTR_DB2ZLOAD_END: u32 = 3040;
pub const SQL_ATTR_DB2ZLOAD_FILETYPE: u32 = 3041;
pub const SQL_ATTR_DB2ZLOAD_MSGFILE: u32 = 3042;
pub const SQL_ATTR_DB2ZLOAD_UTILITYID: u32 = 3043;
pub const SQL_ATTR_CONNECT_PASSIVE: u32 = 3045;
pub const SQL_ATTR_CLIENT_APPLCOMPAT: u32 = 3046;
pub const SQL_ATTR_DB2ZLOAD_LOADFILE: u32 = 3047;
pub const SQL_ATTR_PREFETCH_NROWS: u32 = 3048;
pub const SQL_ATTR_LOB_FILE_THRESHOLD: u32 = 3050;
pub const SQL_ATTR_TRUSTED_CONTEXT_ACCESSTOKEN: u32 = 3051;
pub const SQL_ATTR_CLIENT_USERID: u32 = 1281;
pub const SQL_ATTR_CLIENT_WRKSTNNAME: u32 = 1282;
pub const SQL_ATTR_CLIENT_APPLNAME: u32 = 1283;
pub const SQL_ATTR_CLIENT_ACCTSTR: u32 = 1284;
pub const SQL_ATTR_CLIENT_PROGINFO: u32 = 2516;
pub const SQL_DM_DROP_MODULE: u32 = 1;
pub const SQL_DM_RESTRICT: u32 = 2;
pub const SQL_MU_PROCEDURE_INVOCATION: u32 = 1;
pub const SQL_CM_CREATE_MODULE: u32 = 1;
pub const SQL_CM_AUTHORIZATION: u32 = 2;
pub const SQL_ATTR_WCHARTYPE: u32 = 1252;
pub const SQL_ATTR_LONGDATA_COMPAT: u32 = 1253;
pub const SQL_ATTR_CURRENT_SCHEMA: u32 = 1254;
pub const SQL_ATTR_DB2EXPLAIN: u32 = 1258;
pub const SQL_ATTR_DB2ESTIMATE: u32 = 1259;
pub const SQL_ATTR_PARAMOPT_ATOMIC: u32 = 1260;
pub const SQL_ATTR_STMTTXN_ISOLATION: u32 = 1261;
pub const SQL_ATTR_MAXCONN: u32 = 1262;
pub const SQL_CONNECTTYPE: u32 = 1255;
pub const SQL_SYNC_POINT: u32 = 1256;
pub const SQL_MINMEMORY_USAGE: u32 = 1263;
pub const SQL_CONN_CONTEXT: u32 = 1269;
pub const SQL_ATTR_INHERIT_NULL_CONNECT: u32 = 1270;
pub const SQL_ATTR_FORCE_CONVERSION_ON_CLIENT: u32 = 1275;
pub const SQL_ATTR_INFO_KEYWORDLIST: u32 = 2500;
pub const SQL_ATTR_DISABLE_SYSPLEX: u32 = 2590;
pub const SQL_ATTR_CONNECTTYPE: u32 = 1255;
pub const SQL_ATTR_SYNC_POINT: u32 = 1256;
pub const SQL_ATTR_MINMEMORY_USAGE: u32 = 1263;
pub const SQL_ATTR_CONN_CONTEXT: u32 = 1269;
pub const SQL_LD_COMPAT_YES: u32 = 1;
pub const SQL_LD_COMPAT_NO: u32 = 0;
pub const SQL_LD_COMPAT_DEFAULT: u32 = 0;
pub const SQL_ATTR_EXTENDEDBIND_COPY: u32 = 1;
pub const SQL_ATTR_EXTENDEDBIND_NOCOPY: u32 = 0;
pub const SQL_ATTR_EXTENDEDBIND_DEFAULT: u32 = 0;
pub const SQL_NC_HIGH: u32 = 0;
pub const SQL_NC_LOW: u32 = 1;
pub const SQL_PARC_BATCH_ENABLE: u32 = 1;
pub const SQL_PARC_BATCH_DISABLE: u32 = 0;
pub const SQL_SQLCODEMAP_NOMAP: u32 = 1;
pub const SQL_SQLCODEMAP_MAP: u32 = 2;
pub const SQL_CONNECT_PASSIVE_YES: u32 = 1;
pub const SQL_CONNECT_PASSIVE_NO: u32 = 0;
pub const SQL_CONNECT_PASSIVE_DEFAULT: u32 = 0;
pub const CLI_MAX_LONGVARCHAR: u32 = 1250;
pub const CLI_MAX_VARCHAR: u32 = 1251;
pub const CLI_MAX_CHAR: u32 = 1252;
pub const CLI_MAX_LONGVARGRAPHIC: u32 = 1253;
pub const CLI_MAX_VARGRAPHIC: u32 = 1254;
pub const CLI_MAX_GRAPHIC: u32 = 1255;
pub const SQL_DIAG_MESSAGE_TEXT_PTR: u32 = 2456;
pub const SQL_DIAG_LINE_NUMBER: u32 = 2461;
pub const SQL_DIAG_ERRMC: u32 = 2467;
pub const SQL_DIAG_SQLCA: u32 = 3037;
pub const SQL_DIAG_BYTES_PROCESSED: u32 = 2477;
pub const SQL_DIAG_RELATIVE_COST_ESTIMATE: u32 = 2504;
pub const SQL_DIAG_ROW_COUNT_ESTIMATE: u32 = 2507;
pub const SQL_DIAG_ELAPSED_SERVER_TIME: u32 = 2538;
pub const SQL_DIAG_ELAPSED_NETWORK_TIME: u32 = 2539;
pub const SQL_DIAG_ACCUMULATED_SERVER_TIME: u32 = 2540;
pub const SQL_DIAG_ACCUMULATED_NETWORK_TIME: u32 = 2541;
pub const SQL_DIAG_QUIESCE: u32 = 2549;
pub const SQL_DIAG_TOLERATED_ERROR: u32 = 2559;
pub const SQL_DIAG_NETWORK_STATISTICS: u32 = 2560;
pub const SQL_DIAG_QUIESCE_NO: u32 = 0;
pub const SQL_DIAG_QUIESCE_DATABASE: u32 = 1;
pub const SQL_DIAG_QUIESCE_INSTANCE: u32 = 2;
pub const SQL_ATTR_LITTLE_ENDIAN_UNICODE: u32 = 2457;
pub const SQL_ATTR_DIAGLEVEL: u32 = 2574;
pub const SQL_ATTR_NOTIFYLEVEL: u32 = 2575;
pub const SQL_ATTR_DIAGPATH: u32 = 2576;
pub const SQL_ATTR_MESSAGE_LINE_LENGTH: u32 = 2580;
pub const SQL_ATTR_ENABLE_IFXENV: u32 = 2585;
pub const SQL_ATTR_TRACENOHEADER: u32 = 2598;
pub const SQL_ATTR_DB2TRC_STARTUP_SIZE: u32 = 3019;
pub const SQL_ATOMIC_YES: u32 = 1;
pub const SQL_ATOMIC_NO: u32 = 0;
pub const SQL_ATOMIC_DEFAULT: u32 = 1;
pub const SQL_CONCURRENT_TRANS: u32 = 1;
pub const SQL_COORDINATED_TRANS: u32 = 2;
pub const SQL_CONNECTTYPE_DEFAULT: u32 = 1;
pub const SQL_ONEPHASE: u32 = 1;
pub const SQL_TWOPHASE: u32 = 2;
pub const SQL_SYNCPOINT_DEFAULT: u32 = 1;
pub const SQL_DB2ESTIMATE_ON: u32 = 1;
pub const SQL_DB2ESTIMATE_OFF: u32 = 0;
pub const SQL_DB2ESTIMATE_DEFAULT: u32 = 0;
pub const SQL_DB2EXPLAIN_OFF: u32 = 0;
pub const SQL_DB2EXPLAIN_SNAPSHOT_ON: u32 = 1;
pub const SQL_DB2EXPLAIN_MODE_ON: u32 = 2;
pub const SQL_DB2EXPLAIN_SNAPSHOT_MODE_ON: u32 = 3;
pub const SQL_DB2EXPLAIN_ON: u32 = 1;
pub const SQL_DB2EXPLAIN_DEFAULT: u32 = 0;
pub const SQL_WCHARTYPE_NOCONVERT: u32 = 0;
pub const SQL_WCHARTYPE_DEFAULT: u32 = 0;
pub const SQL_OPTIMIZE_SQLCOLUMNS_OFF: u32 = 0;
pub const SQL_OPTIMIZE_SQLCOLUMNS_ON: u32 = 1;
pub const SQL_OPTIMIZE_SQLCOLUMNS_DEFAULT: u32 = 0;
pub const SQL_CONNECT_WITH_XA_OFF: u32 = 0;
pub const SQL_CONNECT_WITH_XA_ON: u32 = 1;
pub const SQL_CONNECT_WITH_XA_DEFAULT: u32 = 0;
pub const SQL_ATTR_SERVER_MSGTXT_MASK_LOCAL_FIRST: u32 = 0;
pub const SQL_ATTR_SERVER_MSGTXT_MASK_WARNINGS: u32 = 1;
pub const SQL_ATTR_SERVER_MSGTXT_MASK_ERRORS: u32 = 4294967294;
pub const SQL_ATTR_SERVER_MSGTXT_MASK_ALL: u32 = 4294967295;
pub const SQL_ATTR_SERVER_MSGTXT_MASK_DEFAULT: u32 = 0;
pub const SQL_ATTR_QUERY_PATROLLER_DISABLE: u32 = 1;
pub const SQL_ATTR_QUERY_PATROLLER_ENABLE: u32 = 2;
pub const SQL_ATTR_QUERY_PATROLLER_BYPASS: u32 = 3;
pub const SQL_STATICMODE_DISABLED: u32 = 0;
pub const SQL_STATICMODE_CAPTURE: u32 = 1;
pub const SQL_STATICMODE_MATCH: u32 = 2;
pub const SQL_ATTR_DB2_MESSAGE_PREFIX_OFF: u32 = 0;
pub const SQL_ATTR_DB2_MESSAGE_PREFIX_ON: u32 = 1;
pub const SQL_ATTR_DB2_MESSAGE_PREFIX_DEFAULT: u32 = 1;
pub const SQL_ATTR_INSERT_BUFFERING_OFF: u32 = 0;
pub const SQL_ATTR_INSERT_BUFFERING_ON: u32 = 1;
pub const SQL_ATTR_INSERT_BUFFERING_IGD: u32 = 2;
pub const SQL_ROWCOUNT_PREFETCH_OFF: u32 = 0;
pub const SQL_ROWCOUNT_PREFETCH_ON: u32 = 1;
pub const SQL_SCOPE_CURROW: u32 = 0;
pub const SQL_SCOPE_TRANSACTION: u32 = 1;
pub const SQL_SCOPE_SESSION: u32 = 2;
pub const SQL_INDEX_UNIQUE: u32 = 0;
pub const SQL_INDEX_ALL: u32 = 1;
pub const SQL_INDEX_CLUSTERED: u32 = 1;
pub const SQL_INDEX_HASHED: u32 = 2;
pub const SQL_INDEX_OTHER: u32 = 3;
pub const SQL_PC_UNKNOWN: u32 = 0;
pub const SQL_PC_NON_PSEUDO: u32 = 1;
pub const SQL_PC_PSEUDO: u32 = 2;
pub const SQL_ROW_IDENTIFIER: u32 = 1;
pub const SQL_MAPGRAPHIC_DEFAULT: i32 = -1;
pub const SQL_MAPGRAPHIC_GRAPHIC: u32 = 0;
pub const SQL_MAPGRAPHIC_WCHAR: u32 = 1;
pub const SQL_MAPCHAR_DEFAULT: u32 = 0;
pub const SQL_MAPCHAR_WCHAR: u32 = 1;
pub const SQL_FETCH_NEXT: u32 = 1;
pub const SQL_FETCH_FIRST: u32 = 2;
pub const SQL_FETCH_LAST: u32 = 3;
pub const SQL_FETCH_PRIOR: u32 = 4;
pub const SQL_FETCH_ABSOLUTE: u32 = 5;
pub const SQL_FETCH_RELATIVE: u32 = 6;
pub const SQL_EXTENDED_INDICATOR_NOT_SET: u32 = 0;
pub const SQL_EXTENDED_INDICATOR_ENABLE: u32 = 1;
pub const SQL_EXTENDED_INDICATOR_DISABLE: u32 = 2;
pub const SQL_COLUMNWISE_MRI_ON: u32 = 1;
pub const SQL_COLUMNWISE_MRI_OFF: u32 = 0;
pub const SQL_ISREADONLYSQL_YES: u32 = 1;
pub const SQL_ISREADONLYSQL_NO: u32 = 0;
pub const SQL_FREE_MEMORY_ON_STMTCLOSE_YES: u32 = 1;
pub const SQL_FREE_MEMORY_ON_STMTCLOSE_NO: u32 = 0;
pub const SQL_ATTR_CACHE_USRLIBL_YES: u32 = 0;
pub const SQL_ATTR_CACHE_USRLIBL_NO: u32 = 1;
pub const SQL_ATTR_CACHE_USRLIBL_REFRESH: u32 = 2;
pub const SQL_IBMi_FMT_ISO: u32 = 1;
pub const SQL_IBMi_FMT_USA: u32 = 2;
pub const SQL_IBMi_FMT_EUR: u32 = 3;
pub const SQL_IBMi_FMT_JIS: u32 = 4;
pub const SQL_IBMi_FMT_MDY: u32 = 5;
pub const SQL_IBMi_FMT_DMY: u32 = 6;
pub const SQL_IBMi_FMT_YMD: u32 = 7;
pub const SQL_IBMi_FMT_JUL: u32 = 8;
pub const SQL_IBMi_FMT_HMS: u32 = 9;
pub const SQL_IBMi_FMT_JOB: u32 = 10;
pub const SQL_SEP_SLASH: u32 = 1;
pub const SQL_SEP_DASH: u32 = 2;
pub const SQL_SEP_PERIOD: u32 = 3;
pub const SQL_SEP_COMMA: u32 = 4;
pub const SQL_SEP_BLANK: u32 = 5;
pub const SQL_SEP_COLON: u32 = 6;
pub const SQL_SEP_JOB: u32 = 7;
pub const SQL_XML_DECLARATION_NONE: u32 = 0;
pub const SQL_XML_DECLARATION_BOM: u32 = 1;
pub const SQL_XML_DECLARATION_BASE: u32 = 2;
pub const SQL_XML_DECLARATION_ENCATTR: u32 = 4;
pub const SQL_DB2ZLOAD_RECDELIM_NONE: u32 = 0;
pub const SQL_DB2ZLOAD_RECDELIM_ALF: u32 = 1;
pub const SQL_DB2ZLOAD_RECDELIM_ENL: u32 = 2;
pub const SQL_DB2ZLOAD_RECDELIM_CRLF: u32 = 3;
pub const SQL_DB2ZLOAD_FILETYPE_DEL: u32 = 1;
pub const SQL_DB2ZLOAD_FILETYPE_INT: u32 = 2;
pub const SQL_DB2ZLOAD_FILETYPE_SPANNED: u32 = 3;
pub const DSD_ACR_AFFINITY: u32 = 1;
pub const SQL_ATTR_OUTPUT_NTS: u32 = 10001;
pub const SQL_FILE_READ: u32 = 2;
pub const SQL_FILE_CREATE: u32 = 8;
pub const SQL_FILE_OVERWRITE: u32 = 16;
pub const SQL_FILE_APPEND: u32 = 32;
pub const SQL_FROM_LOCATOR: u32 = 2;
pub const SQL_FROM_LITERAL: u32 = 3;
pub const SQL_ROUND_HALF_EVEN: u32 = 0;
pub const SQL_ROUND_HALF_UP: u32 = 1;
pub const SQL_ROUND_DOWN: u32 = 2;
pub const SQL_ROUND_CEILING: u32 = 3;
pub const SQL_ROUND_FLOOR: u32 = 4;
pub const SQL_NETWORK_STATISTICS_ON_SKIP_NOSERVER: u32 = 2;
pub const SQL_NETWORK_STATISTICS_ON: u32 = 1;
pub const SQL_NETWORK_STATISTICS_OFF: u32 = 0;
pub const SQL_NETWORK_STATISTICS_DEFAULT: u32 = 0;
pub const SQL_READ_ONLY_CONNECTION_ON: u32 = 1;
pub const SQL_READ_ONLY_CONNECTION_OFF: u32 = 0;
pub const SQL_READ_ONLY_CONNECTION_DEFAULT: u32 = 0;
pub const SQL_UNASSIGNED: i32 = -7;
pub const SQL_DETECT_READ_ONLY_TXN_ENABLE: u32 = 1;
pub const SQL_DETECT_READ_ONLY_TXN_DISABLE: u32 = 0;
pub const SQL_C_WCHAR: i32 = -8;
pub const SQL_C_TCHAR: u32 = 1;

#[doc = " Define fixed size integer types."]
pub type sqlint8 = ::std::os::raw::c_char;
pub type sqluint8 = ::std::os::raw::c_uchar;
pub type sqlint16 = ::std::os::raw::c_short;
pub type sqluint16 = ::std::os::raw::c_ushort;
pub type sqlint32 = ::std::os::raw::c_int;
pub type sqluint32 = ::std::os::raw::c_uint;
pub type sqlint64 = ::std::os::raw::c_long;
pub type sqluint64 = ::std::os::raw::c_ulong;
pub type sqlintptr = sqlint64;
pub type sqluintptr = sqluint64;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct sqlca {
    pub sqlcaid: [::std::os::raw::c_char; 8usize],
    pub sqlcabc: sqlint32,
    pub sqlcode: sqlint32,
    pub sqlerrml: ::std::os::raw::c_short,
    pub sqlerrmc: [::std::os::raw::c_char; 70usize],
    pub sqlerrp: [::std::os::raw::c_char; 8usize],
    pub sqlerrd: [sqlint32; 6usize],
    pub sqlwarn: [::std::os::raw::c_char; 11usize],
    pub sqlstate: [::std::os::raw::c_char; 5usize],
}

pub type size_t = ::std::os::raw::c_ulong;
pub type wchar_t = ::std::os::raw::c_int;
pub const idtype_t_P_ALL: idtype_t = 0;
pub const idtype_t_P_PID: idtype_t = 1;
pub const idtype_t_P_PGID: idtype_t = 2;
pub type idtype_t = ::std::os::raw::c_uint;
pub type _Float32 = f32;
pub type _Float64 = f64;
pub type _Float32x = f64;
pub type _Float64x = u128;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct div_t {
    pub quot: ::std::os::raw::c_int,
    pub rem: ::std::os::raw::c_int,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ldiv_t {
    pub quot: ::std::os::raw::c_long,
    pub rem: ::std::os::raw::c_long,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct lldiv_t {
    pub quot: ::std::os::raw::c_longlong,
    pub rem: ::std::os::raw::c_longlong,
}

extern "C" {
    pub fn __ctype_get_mb_cur_max() -> size_t;
}
extern "C" {
    pub fn atof(__nptr: *const ::std::os::raw::c_char) -> f64;
}
extern "C" {
    pub fn atoi(__nptr: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn atol(__nptr: *const ::std::os::raw::c_char) -> ::std::os::raw::c_long;
}
extern "C" {
    pub fn atoll(__nptr: *const ::std::os::raw::c_char) -> ::std::os::raw::c_longlong;
}
extern "C" {
    pub fn strtod(
        __nptr: *const ::std::os::raw::c_char,
        __endptr: *mut *mut ::std::os::raw::c_char,
    ) -> f64;
}
extern "C" {
    pub fn strtof(
        __nptr: *const ::std::os::raw::c_char,
        __endptr: *mut *mut ::std::os::raw::c_char,
    ) -> f32;
}
extern "C" {
    pub fn strtold(
        __nptr: *const ::std::os::raw::c_char,
        __endptr: *mut *mut ::std::os::raw::c_char,
    ) -> u128;
}
extern "C" {
    pub fn strtol(
        __nptr: *const ::std::os::raw::c_char,
        __endptr: *mut *mut ::std::os::raw::c_char,
        __base: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_long;
}
extern "C" {
    pub fn strtoul(
        __nptr: *const ::std::os::raw::c_char,
        __endptr: *mut *mut ::std::os::raw::c_char,
        __base: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_ulong;
}
extern "C" {
    pub fn strtoq(
        __nptr: *const ::std::os::raw::c_char,
        __endptr: *mut *mut ::std::os::raw::c_char,
        __base: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_longlong;
}
extern "C" {
    pub fn strtouq(
        __nptr: *const ::std::os::raw::c_char,
        __endptr: *mut *mut ::std::os::raw::c_char,
        __base: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_ulonglong;
}
extern "C" {
    pub fn strtoll(
        __nptr: *const ::std::os::raw::c_char,
        __endptr: *mut *mut ::std::os::raw::c_char,
        __base: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_longlong;
}
extern "C" {
    pub fn strtoull(
        __nptr: *const ::std::os::raw::c_char,
        __endptr: *mut *mut ::std::os::raw::c_char,
        __base: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_ulonglong;
}
extern "C" {
    pub fn l64a(__n: ::std::os::raw::c_long) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn a64l(__s: *const ::std::os::raw::c_char) -> ::std::os::raw::c_long;
}
pub type __u_char = ::std::os::raw::c_uchar;
pub type __u_short = ::std::os::raw::c_ushort;
pub type __u_int = ::std::os::raw::c_uint;
pub type __u_long = ::std::os::raw::c_ulong;
pub type __int8_t = ::std::os::raw::c_schar;
pub type __uint8_t = ::std::os::raw::c_uchar;
pub type __int16_t = ::std::os::raw::c_short;
pub type __uint16_t = ::std::os::raw::c_ushort;
pub type __int32_t = ::std::os::raw::c_int;
pub type __uint32_t = ::std::os::raw::c_uint;
pub type __int64_t = ::std::os::raw::c_long;
pub type __uint64_t = ::std::os::raw::c_ulong;
pub type __quad_t = ::std::os::raw::c_long;
pub type __u_quad_t = ::std::os::raw::c_ulong;
pub type __intmax_t = ::std::os::raw::c_long;
pub type __uintmax_t = ::std::os::raw::c_ulong;
pub type __dev_t = ::std::os::raw::c_ulong;
pub type __uid_t = ::std::os::raw::c_uint;
pub type __gid_t = ::std::os::raw::c_uint;
pub type __ino_t = ::std::os::raw::c_ulong;
pub type __ino64_t = ::std::os::raw::c_ulong;
pub type __mode_t = ::std::os::raw::c_uint;
pub type __nlink_t = ::std::os::raw::c_ulong;
pub type __off_t = ::std::os::raw::c_long;
pub type __off64_t = ::std::os::raw::c_long;
pub type __pid_t = ::std::os::raw::c_int;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __fsid_t {
    pub __val: [::std::os::raw::c_int; 2usize],
}

pub type __clock_t = ::std::os::raw::c_long;
pub type __rlim_t = ::std::os::raw::c_ulong;
pub type __rlim64_t = ::std::os::raw::c_ulong;
pub type __id_t = ::std::os::raw::c_uint;
pub type __time_t = ::std::os::raw::c_long;
pub type __useconds_t = ::std::os::raw::c_uint;
pub type __suseconds_t = ::std::os::raw::c_long;
pub type __daddr_t = ::std::os::raw::c_int;
pub type __key_t = ::std::os::raw::c_int;
pub type __clockid_t = ::std::os::raw::c_int;
pub type __timer_t = *mut ::std::os::raw::c_void;
pub type __blksize_t = ::std::os::raw::c_long;
pub type __blkcnt_t = ::std::os::raw::c_long;
pub type __blkcnt64_t = ::std::os::raw::c_long;
pub type __fsblkcnt_t = ::std::os::raw::c_ulong;
pub type __fsblkcnt64_t = ::std::os::raw::c_ulong;
pub type __fsfilcnt_t = ::std::os::raw::c_ulong;
pub type __fsfilcnt64_t = ::std::os::raw::c_ulong;
pub type __fsword_t = ::std::os::raw::c_long;
pub type __ssize_t = ::std::os::raw::c_long;
pub type __syscall_slong_t = ::std::os::raw::c_long;
pub type __syscall_ulong_t = ::std::os::raw::c_ulong;
pub type __loff_t = __off64_t;
pub type __caddr_t = *mut ::std::os::raw::c_char;
pub type __intptr_t = ::std::os::raw::c_long;
pub type __socklen_t = ::std::os::raw::c_uint;
pub type __sig_atomic_t = ::std::os::raw::c_int;
pub type u_char = __u_char;
pub type u_short = __u_short;
pub type u_int = __u_int;
pub type u_long = __u_long;
pub type quad_t = __quad_t;
pub type u_quad_t = __u_quad_t;
pub type fsid_t = __fsid_t;
pub type loff_t = __loff_t;
pub type ino_t = __ino_t;
pub type dev_t = __dev_t;
pub type gid_t = __gid_t;
pub type mode_t = __mode_t;
pub type nlink_t = __nlink_t;
pub type uid_t = __uid_t;
pub type off_t = __off_t;
pub type pid_t = __pid_t;
pub type id_t = __id_t;
pub type ssize_t = __ssize_t;
pub type daddr_t = __daddr_t;
pub type caddr_t = __caddr_t;
pub type key_t = __key_t;
pub type clock_t = __clock_t;
pub type clockid_t = __clockid_t;
pub type time_t = __time_t;
pub type timer_t = __timer_t;
pub type ulong = ::std::os::raw::c_ulong;
pub type ushort = ::std::os::raw::c_ushort;
pub type uint = ::std::os::raw::c_uint;
pub type u_int8_t = ::std::os::raw::c_uchar;
pub type u_int16_t = ::std::os::raw::c_ushort;
pub type u_int32_t = ::std::os::raw::c_uint;
pub type u_int64_t = ::std::os::raw::c_ulong;
pub type register_t = ::std::os::raw::c_long;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __sigset_t {
    pub __val: [::std::os::raw::c_ulong; 16usize],
}

pub type sigset_t = __sigset_t;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct timeval {
    pub tv_sec: __time_t,
    pub tv_usec: __suseconds_t,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct timespec {
    pub tv_sec: __time_t,
    pub tv_nsec: __syscall_slong_t,
}

pub type suseconds_t = __suseconds_t;
pub type __fd_mask = ::std::os::raw::c_long;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct fd_set {
    pub __fds_bits: [__fd_mask; 16usize],
}

pub type fd_mask = __fd_mask;
extern "C" {
    pub fn select(
        __nfds: ::std::os::raw::c_int,
        __readfds: *mut fd_set,
        __writefds: *mut fd_set,
        __exceptfds: *mut fd_set,
        __timeout: *mut timeval,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn pselect(
        __nfds: ::std::os::raw::c_int,
        __readfds: *mut fd_set,
        __writefds: *mut fd_set,
        __exceptfds: *mut fd_set,
        __timeout: *const timespec,
        __sigmask: *const __sigset_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn gnu_dev_major(__dev: __dev_t) -> ::std::os::raw::c_uint;
}
extern "C" {
    pub fn gnu_dev_minor(__dev: __dev_t) -> ::std::os::raw::c_uint;
}
extern "C" {
    pub fn gnu_dev_makedev(
        __major: ::std::os::raw::c_uint,
        __minor: ::std::os::raw::c_uint,
    ) -> __dev_t;
}
pub type blksize_t = __blksize_t;
pub type blkcnt_t = __blkcnt_t;
pub type fsblkcnt_t = __fsblkcnt_t;
pub type fsfilcnt_t = __fsfilcnt_t;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __pthread_rwlock_arch_t {
    pub __readers: ::std::os::raw::c_uint,
    pub __writers: ::std::os::raw::c_uint,
    pub __wrphase_futex: ::std::os::raw::c_uint,
    pub __writers_futex: ::std::os::raw::c_uint,
    pub __pad3: ::std::os::raw::c_uint,
    pub __pad4: ::std::os::raw::c_uint,
    pub __cur_writer: ::std::os::raw::c_int,
    pub __shared: ::std::os::raw::c_int,
    pub __rwelision: ::std::os::raw::c_schar,
    pub __pad1: [::std::os::raw::c_uchar; 7usize],
    pub __pad2: ::std::os::raw::c_ulong,
    pub __flags: ::std::os::raw::c_uint,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __pthread_internal_list {
    pub __prev: *mut __pthread_internal_list,
    pub __next: *mut __pthread_internal_list,
}

pub type __pthread_list_t = __pthread_internal_list;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __pthread_mutex_s {
    pub __lock: ::std::os::raw::c_int,
    pub __count: ::std::os::raw::c_uint,
    pub __owner: ::std::os::raw::c_int,
    pub __nusers: ::std::os::raw::c_uint,
    pub __kind: ::std::os::raw::c_int,
    pub __spins: ::std::os::raw::c_short,
    pub __elision: ::std::os::raw::c_short,
    pub __list: __pthread_list_t,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct __pthread_cond_s {
    pub __bindgen_anon_1: __pthread_cond_s__bindgen_ty_1,
    pub __bindgen_anon_2: __pthread_cond_s__bindgen_ty_2,
    pub __g_refs: [::std::os::raw::c_uint; 2usize],
    pub __g_size: [::std::os::raw::c_uint; 2usize],
    pub __g1_orig_size: ::std::os::raw::c_uint,
    pub __wrefs: ::std::os::raw::c_uint,
    pub __g_signals: [::std::os::raw::c_uint; 2usize],
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union __pthread_cond_s__bindgen_ty_1 {
    pub __wseq: ::std::os::raw::c_ulonglong,
    pub __wseq32: __pthread_cond_s__bindgen_ty_1__bindgen_ty_1,
    _bindgen_union_align: u64,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __pthread_cond_s__bindgen_ty_1__bindgen_ty_1 {
    pub __low: ::std::os::raw::c_uint,
    pub __high: ::std::os::raw::c_uint,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union __pthread_cond_s__bindgen_ty_2 {
    pub __g1_start: ::std::os::raw::c_ulonglong,
    pub __g1_start32: __pthread_cond_s__bindgen_ty_2__bindgen_ty_1,
    _bindgen_union_align: u64,
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct __pthread_cond_s__bindgen_ty_2__bindgen_ty_1 {
    pub __low: ::std::os::raw::c_uint,
    pub __high: ::std::os::raw::c_uint,
}

pub type pthread_t = ::std::os::raw::c_ulong;

#[repr(C)]
#[derive(Copy, Clone)]
pub union pthread_mutexattr_t {
    pub __size: [::std::os::raw::c_char; 4usize],
    pub __align: ::std::os::raw::c_int,
    _bindgen_union_align: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union pthread_condattr_t {
    pub __size: [::std::os::raw::c_char; 4usize],
    pub __align: ::std::os::raw::c_int,
    _bindgen_union_align: u32,
}

pub type pthread_key_t = ::std::os::raw::c_uint;
pub type pthread_once_t = ::std::os::raw::c_int;

#[repr(C)]
#[derive(Copy, Clone)]
pub union pthread_attr_t {
    pub __size: [::std::os::raw::c_char; 56usize],
    pub __align: ::std::os::raw::c_long,
    _bindgen_union_align: [u64; 7usize],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union pthread_mutex_t {
    pub __data: __pthread_mutex_s,
    pub __size: [::std::os::raw::c_char; 40usize],
    pub __align: ::std::os::raw::c_long,
    _bindgen_union_align: [u64; 5usize],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union pthread_cond_t {
    pub __data: __pthread_cond_s,
    pub __size: [::std::os::raw::c_char; 48usize],
    pub __align: ::std::os::raw::c_longlong,
    _bindgen_union_align: [u64; 6usize],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union pthread_rwlock_t {
    pub __data: __pthread_rwlock_arch_t,
    pub __size: [::std::os::raw::c_char; 56usize],
    pub __align: ::std::os::raw::c_long,
    _bindgen_union_align: [u64; 7usize],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union pthread_rwlockattr_t {
    pub __size: [::std::os::raw::c_char; 8usize],
    pub __align: ::std::os::raw::c_long,
    _bindgen_union_align: u64,
}

pub type pthread_spinlock_t = ::std::os::raw::c_int;

#[repr(C)]
#[derive(Copy, Clone)]
pub union pthread_barrier_t {
    pub __size: [::std::os::raw::c_char; 32usize],
    pub __align: ::std::os::raw::c_long,
    _bindgen_union_align: [u64; 4usize],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union pthread_barrierattr_t {
    pub __size: [::std::os::raw::c_char; 4usize],
    pub __align: ::std::os::raw::c_int,
    _bindgen_union_align: u32,
}

extern "C" {
    pub fn random() -> ::std::os::raw::c_long;
}
extern "C" {
    pub fn srandom(__seed: ::std::os::raw::c_uint);
}
extern "C" {
    pub fn initstate(
        __seed: ::std::os::raw::c_uint,
        __statebuf: *mut ::std::os::raw::c_char,
        __statelen: size_t,
    ) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn setstate(__statebuf: *mut ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char;
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct random_data {
    pub fptr: *mut i32,
    pub rptr: *mut i32,
    pub state: *mut i32,
    pub rand_type: ::std::os::raw::c_int,
    pub rand_deg: ::std::os::raw::c_int,
    pub rand_sep: ::std::os::raw::c_int,
    pub end_ptr: *mut i32,
}

extern "C" {
    pub fn random_r(__buf: *mut random_data, __result: *mut i32) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn srandom_r(
        __seed: ::std::os::raw::c_uint,
        __buf: *mut random_data,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn initstate_r(
        __seed: ::std::os::raw::c_uint,
        __statebuf: *mut ::std::os::raw::c_char,
        __statelen: size_t,
        __buf: *mut random_data,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn setstate_r(
        __statebuf: *mut ::std::os::raw::c_char,
        __buf: *mut random_data,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn rand() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn srand(__seed: ::std::os::raw::c_uint);
}
extern "C" {
    pub fn rand_r(__seed: *mut ::std::os::raw::c_uint) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn drand48() -> f64;
}
extern "C" {
    pub fn erand48(__xsubi: *mut ::std::os::raw::c_ushort) -> f64;
}
extern "C" {
    pub fn lrand48() -> ::std::os::raw::c_long;
}
extern "C" {
    pub fn nrand48(__xsubi: *mut ::std::os::raw::c_ushort) -> ::std::os::raw::c_long;
}
extern "C" {
    pub fn mrand48() -> ::std::os::raw::c_long;
}
extern "C" {
    pub fn jrand48(__xsubi: *mut ::std::os::raw::c_ushort) -> ::std::os::raw::c_long;
}
extern "C" {
    pub fn srand48(__seedval: ::std::os::raw::c_long);
}
extern "C" {
    pub fn seed48(__seed16v: *mut ::std::os::raw::c_ushort) -> *mut ::std::os::raw::c_ushort;
}
extern "C" {
    pub fn lcong48(__param: *mut ::std::os::raw::c_ushort);
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct drand48_data {
    pub __x: [::std::os::raw::c_ushort; 3usize],
    pub __old_x: [::std::os::raw::c_ushort; 3usize],
    pub __c: ::std::os::raw::c_ushort,
    pub __init: ::std::os::raw::c_ushort,
    pub __a: ::std::os::raw::c_ulonglong,
}

extern "C" {
    pub fn drand48_r(__buffer: *mut drand48_data, __result: *mut f64) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn erand48_r(
        __xsubi: *mut ::std::os::raw::c_ushort,
        __buffer: *mut drand48_data,
        __result: *mut f64,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn lrand48_r(
        __buffer: *mut drand48_data,
        __result: *mut ::std::os::raw::c_long,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn nrand48_r(
        __xsubi: *mut ::std::os::raw::c_ushort,
        __buffer: *mut drand48_data,
        __result: *mut ::std::os::raw::c_long,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn mrand48_r(
        __buffer: *mut drand48_data,
        __result: *mut ::std::os::raw::c_long,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn jrand48_r(
        __xsubi: *mut ::std::os::raw::c_ushort,
        __buffer: *mut drand48_data,
        __result: *mut ::std::os::raw::c_long,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn srand48_r(
        __seedval: ::std::os::raw::c_long,
        __buffer: *mut drand48_data,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn seed48_r(
        __seed16v: *mut ::std::os::raw::c_ushort,
        __buffer: *mut drand48_data,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn lcong48_r(
        __param: *mut ::std::os::raw::c_ushort,
        __buffer: *mut drand48_data,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn malloc(__size: ::std::os::raw::c_ulong) -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn calloc(
        __nmemb: ::std::os::raw::c_ulong,
        __size: ::std::os::raw::c_ulong,
    ) -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn realloc(
        __ptr: *mut ::std::os::raw::c_void,
        __size: ::std::os::raw::c_ulong,
    ) -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn free(__ptr: *mut ::std::os::raw::c_void);
}
extern "C" {
    pub fn alloca(__size: ::std::os::raw::c_ulong) -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn valloc(__size: size_t) -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn posix_memalign(
        __memptr: *mut *mut ::std::os::raw::c_void,
        __alignment: size_t,
        __size: size_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn aligned_alloc(__alignment: size_t, __size: size_t) -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn abort();
}
extern "C" {
    pub fn atexit(__func: ::std::option::Option<unsafe extern "C" fn()>) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn at_quick_exit(
        __func: ::std::option::Option<unsafe extern "C" fn()>,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn on_exit(
        __func: ::std::option::Option<
            unsafe extern "C" fn(
                __status: ::std::os::raw::c_int,
                __arg: *mut ::std::os::raw::c_void,
            ),
        >,
        __arg: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn exit(__status: ::std::os::raw::c_int);
}
extern "C" {
    pub fn quick_exit(__status: ::std::os::raw::c_int);
}
extern "C" {
    pub fn _Exit(__status: ::std::os::raw::c_int);
}
extern "C" {
    pub fn getenv(__name: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn putenv(__string: *mut ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn setenv(
        __name: *const ::std::os::raw::c_char,
        __value: *const ::std::os::raw::c_char,
        __replace: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn unsetenv(__name: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn clearenv() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn mktemp(__template: *mut ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn mkstemp(__template: *mut ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn mkstemps(
        __template: *mut ::std::os::raw::c_char,
        __suffixlen: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn mkdtemp(__template: *mut ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn system(__command: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn realpath(
        __name: *const ::std::os::raw::c_char,
        __resolved: *mut ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_char;
}
pub type __compar_fn_t = ::std::option::Option<
    unsafe extern "C" fn(
        arg1: *const ::std::os::raw::c_void,
        arg2: *const ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int,
>;
extern "C" {
    pub fn bsearch(
        __key: *const ::std::os::raw::c_void,
        __base: *const ::std::os::raw::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    ) -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn qsort(
        __base: *mut ::std::os::raw::c_void,
        __nmemb: size_t,
        __size: size_t,
        __compar: __compar_fn_t,
    );
}
extern "C" {
    pub fn abs(__x: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn labs(__x: ::std::os::raw::c_long) -> ::std::os::raw::c_long;
}
extern "C" {
    pub fn llabs(__x: ::std::os::raw::c_longlong) -> ::std::os::raw::c_longlong;
}
extern "C" {
    pub fn div(__numer: ::std::os::raw::c_int, __denom: ::std::os::raw::c_int) -> div_t;
}
extern "C" {
    pub fn ldiv(__numer: ::std::os::raw::c_long, __denom: ::std::os::raw::c_long) -> ldiv_t;
}
extern "C" {
    pub fn lldiv(
        __numer: ::std::os::raw::c_longlong,
        __denom: ::std::os::raw::c_longlong,
    ) -> lldiv_t;
}
extern "C" {
    pub fn ecvt(
        __value: f64,
        __ndigit: ::std::os::raw::c_int,
        __decpt: *mut ::std::os::raw::c_int,
        __sign: *mut ::std::os::raw::c_int,
    ) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn fcvt(
        __value: f64,
        __ndigit: ::std::os::raw::c_int,
        __decpt: *mut ::std::os::raw::c_int,
        __sign: *mut ::std::os::raw::c_int,
    ) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn gcvt(
        __value: f64,
        __ndigit: ::std::os::raw::c_int,
        __buf: *mut ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn qecvt(
        __value: u128,
        __ndigit: ::std::os::raw::c_int,
        __decpt: *mut ::std::os::raw::c_int,
        __sign: *mut ::std::os::raw::c_int,
    ) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn qfcvt(
        __value: u128,
        __ndigit: ::std::os::raw::c_int,
        __decpt: *mut ::std::os::raw::c_int,
        __sign: *mut ::std::os::raw::c_int,
    ) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn qgcvt(
        __value: u128,
        __ndigit: ::std::os::raw::c_int,
        __buf: *mut ::std::os::raw::c_char,
    ) -> *mut ::std::os::raw::c_char;
}
extern "C" {
    pub fn ecvt_r(
        __value: f64,
        __ndigit: ::std::os::raw::c_int,
        __decpt: *mut ::std::os::raw::c_int,
        __sign: *mut ::std::os::raw::c_int,
        __buf: *mut ::std::os::raw::c_char,
        __len: size_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn fcvt_r(
        __value: f64,
        __ndigit: ::std::os::raw::c_int,
        __decpt: *mut ::std::os::raw::c_int,
        __sign: *mut ::std::os::raw::c_int,
        __buf: *mut ::std::os::raw::c_char,
        __len: size_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn qecvt_r(
        __value: u128,
        __ndigit: ::std::os::raw::c_int,
        __decpt: *mut ::std::os::raw::c_int,
        __sign: *mut ::std::os::raw::c_int,
        __buf: *mut ::std::os::raw::c_char,
        __len: size_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn qfcvt_r(
        __value: u128,
        __ndigit: ::std::os::raw::c_int,
        __decpt: *mut ::std::os::raw::c_int,
        __sign: *mut ::std::os::raw::c_int,
        __buf: *mut ::std::os::raw::c_char,
        __len: size_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn mblen(__s: *const ::std::os::raw::c_char, __n: size_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn mbtowc(
        __pwc: *mut wchar_t,
        __s: *const ::std::os::raw::c_char,
        __n: size_t,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn wctomb(__s: *mut ::std::os::raw::c_char, __wchar: wchar_t) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn mbstowcs(
        __pwcs: *mut wchar_t,
        __s: *const ::std::os::raw::c_char,
        __n: size_t,
    ) -> size_t;
}
extern "C" {
    pub fn wcstombs(
        __s: *mut ::std::os::raw::c_char,
        __pwcs: *const wchar_t,
        __n: size_t,
    ) -> size_t;
}
extern "C" {
    pub fn rpmatch(__response: *const ::std::os::raw::c_char) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn getsubopt(
        __optionp: *mut *mut ::std::os::raw::c_char,
        __tokens: *const *mut ::std::os::raw::c_char,
        __valuep: *mut *mut ::std::os::raw::c_char,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn getloadavg(__loadavg: *mut f64, __nelem: ::std::os::raw::c_int)
        -> ::std::os::raw::c_int;
}
pub type SCHAR = ::std::os::raw::c_schar;
pub type UCHAR = ::std::os::raw::c_uchar;
pub type SWORD = ::std::os::raw::c_short;
pub type USHORT = ::std::os::raw::c_ushort;
pub type SSHORT = ::std::os::raw::c_short;
pub type UWORD = ::std::os::raw::c_ushort;
pub type SDWORD = sqlint32;
pub type ULONG = sqluint32;
pub type UDWORD = sqluint32;
pub type SLONG = sqlint32;
pub type SDOUBLE = f64;
pub type SFLOAT = f32;
pub type SQLDATE = ::std::os::raw::c_uchar;
pub type SQLTIME = ::std::os::raw::c_uchar;
pub type SQLTIMESTAMP = ::std::os::raw::c_uchar;
pub type SQLDECIMAL = ::std::os::raw::c_uchar;
pub type SQLNUMERIC = ::std::os::raw::c_uchar;
pub type LDOUBLE = f64;
pub type PTR = *mut ::std::os::raw::c_void;
pub type HENV = *mut ::std::os::raw::c_void;
pub type HDBC = *mut ::std::os::raw::c_void;
pub type HSTMT = *mut ::std::os::raw::c_void;
pub type RETCODE = ::std::os::raw::c_short;
pub type SQLCHAR = UCHAR;
pub type SQLVARCHAR = UCHAR;
pub type SQLSCHAR = SCHAR;
pub type SQLINTEGER = SDWORD;
pub type SQLSMALLINT = SWORD;
pub type SQLDOUBLE = SDOUBLE;
pub type SQLFLOAT = SDOUBLE;
pub type SQLREAL = SFLOAT;
pub type SQLRETURN = SQLSMALLINT;
pub type SQLUINTEGER = UDWORD;
pub type SQLUSMALLINT = UWORD;
pub type SQLPOINTER = PTR;
pub type SQLDBCHAR = ::std::os::raw::c_ushort;
pub type SQLWCHAR = ::std::os::raw::c_ushort;
pub type SQLTCHAR = SQLCHAR;
pub type SQLHANDLE = SQLINTEGER;
pub type SQLHENV = SQLINTEGER;
pub type SQLHDBC = SQLINTEGER;
pub type SQLHSTMT = SQLINTEGER;
pub type SQLHWND = SQLPOINTER;
pub type SQLHDESC = SQLHANDLE;
pub type SQLBIGINT = ::std::os::raw::c_long;
pub type SQLUBIGINT = ::std::os::raw::c_ulong;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct DATE_STRUCT {
    pub year: SQLSMALLINT,
    pub month: SQLUSMALLINT,
    pub day: SQLUSMALLINT,
}

pub type SQL_DATE_STRUCT = DATE_STRUCT;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TIME_STRUCT {
    pub hour: SQLUSMALLINT,
    pub minute: SQLUSMALLINT,
    pub second: SQLUSMALLINT,
}

pub type SQL_TIME_STRUCT = TIME_STRUCT;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TIMESTAMP_STRUCT {
    pub year: SQLSMALLINT,
    pub month: SQLUSMALLINT,
    pub day: SQLUSMALLINT,
    pub hour: SQLUSMALLINT,
    pub minute: SQLUSMALLINT,
    pub second: SQLUSMALLINT,
    pub fraction: SQLUINTEGER,
}

pub type SQL_TIMESTAMP_STRUCT = TIMESTAMP_STRUCT;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TIMESTAMP_STRUCT_EXT {
    pub year: SQLSMALLINT,
    pub month: SQLUSMALLINT,
    pub day: SQLUSMALLINT,
    pub hour: SQLUSMALLINT,
    pub minute: SQLUSMALLINT,
    pub second: SQLUSMALLINT,
    pub fraction: SQLUINTEGER,
    pub fraction2: SQLUINTEGER,
}

pub type SQL_TIMESTAMP_STRUCT_EXT = TIMESTAMP_STRUCT_EXT;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TIMESTAMP_STRUCT_EXT_TZ {
    pub year: SQLSMALLINT,
    pub month: SQLUSMALLINT,
    pub day: SQLUSMALLINT,
    pub hour: SQLUSMALLINT,
    pub minute: SQLUSMALLINT,
    pub second: SQLUSMALLINT,
    pub fraction: SQLUINTEGER,
    pub fraction2: SQLUINTEGER,
    pub timezone_hour: SQLSMALLINT,
    pub timezone_minute: SQLSMALLINT,
}

pub type SQL_TIMESTAMP_STRUCT_EXT_TZ = TIMESTAMP_STRUCT_EXT_TZ;
pub const SQLINTERVAL_SQL_IS_YEAR: SQLINTERVAL = 1;
pub const SQLINTERVAL_SQL_IS_MONTH: SQLINTERVAL = 2;
pub const SQLINTERVAL_SQL_IS_DAY: SQLINTERVAL = 3;
pub const SQLINTERVAL_SQL_IS_HOUR: SQLINTERVAL = 4;
pub const SQLINTERVAL_SQL_IS_MINUTE: SQLINTERVAL = 5;
pub const SQLINTERVAL_SQL_IS_SECOND: SQLINTERVAL = 6;
pub const SQLINTERVAL_SQL_IS_YEAR_TO_MONTH: SQLINTERVAL = 7;
pub const SQLINTERVAL_SQL_IS_DAY_TO_HOUR: SQLINTERVAL = 8;
pub const SQLINTERVAL_SQL_IS_DAY_TO_MINUTE: SQLINTERVAL = 9;
pub const SQLINTERVAL_SQL_IS_DAY_TO_SECOND: SQLINTERVAL = 10;
pub const SQLINTERVAL_SQL_IS_HOUR_TO_MINUTE: SQLINTERVAL = 11;
pub const SQLINTERVAL_SQL_IS_HOUR_TO_SECOND: SQLINTERVAL = 12;
pub const SQLINTERVAL_SQL_IS_MINUTE_TO_SECOND: SQLINTERVAL = 13;
pub type SQLINTERVAL = ::std::os::raw::c_uint;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tagSQL_YEAR_MONTH {
    pub year: SQLUINTEGER,
    pub month: SQLUINTEGER,
}

pub type SQL_YEAR_MONTH_STRUCT = tagSQL_YEAR_MONTH;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tagSQL_DAY_SECOND {
    pub day: SQLUINTEGER,
    pub hour: SQLUINTEGER,
    pub minute: SQLUINTEGER,
    pub second: SQLUINTEGER,
    pub fraction: SQLUINTEGER,
}

pub type SQL_DAY_SECOND_STRUCT = tagSQL_DAY_SECOND;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct tagSQL_INTERVAL_STRUCT {
    pub interval_type: SQLINTERVAL,
    pub interval_sign: SQLSMALLINT,
    pub intval: tagSQL_INTERVAL_STRUCT__bindgen_ty_1,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union tagSQL_INTERVAL_STRUCT__bindgen_ty_1 {
    pub year_month: SQL_YEAR_MONTH_STRUCT,
    pub day_second: SQL_DAY_SECOND_STRUCT,
    _bindgen_union_align: [u32; 5usize],
}

pub type SQL_INTERVAL_STRUCT = tagSQL_INTERVAL_STRUCT;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct tagSQL_NUMERIC_STRUCT {
    pub precision: SQLCHAR,
    pub scale: SQLSCHAR,
    pub sign: SQLCHAR,
    pub val: [SQLCHAR; 16usize],
}

pub type SQL_NUMERIC_STRUCT = tagSQL_NUMERIC_STRUCT;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SQLDECIMAL64 {
    pub udec64: SQLDECIMAL64__bindgen_ty_1,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union SQLDECIMAL64__bindgen_ty_1 {
    pub dummy: SQLDOUBLE,
    pub dec64: [SQLCHAR; 8usize],
    _bindgen_union_align: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct SQLDECIMAL128 {
    pub udec128: SQLDECIMAL128__bindgen_ty_1,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union SQLDECIMAL128__bindgen_ty_1 {
    pub dummy: SQLDOUBLE,
    pub dec128: [SQLCHAR; 16usize],
    _bindgen_union_align: [u64; 2usize],
}

extern "C" {
    pub fn SQLAllocConnect(henv: SQLHENV, phdbc: *mut SQLHDBC) -> SQLRETURN;
}
extern "C" {
    pub fn SQLAllocEnv(phenv: *mut SQLHENV) -> SQLRETURN;
}
extern "C" {
    pub fn SQLAllocStmt(hdbc: SQLHDBC, phstmt: *mut SQLHSTMT) -> SQLRETURN;
}
extern "C" {
    pub fn SQLAllocHandle(
        fHandleType: SQLSMALLINT,
        hInput: SQLHANDLE,
        phOutput: *mut SQLHANDLE,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLBindCol(
        hstmt: SQLHSTMT,
        icol: SQLUSMALLINT,
        fCType: SQLSMALLINT,
        rgbValue: SQLPOINTER,
        cbValueMax: SQLINTEGER,
        pcbValue: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLCancel(hstmt: SQLHSTMT) -> SQLRETURN;
}
extern "C" {
    pub fn SQLColAttribute(
        hstmt: SQLHSTMT,
        icol: SQLUSMALLINT,
        fDescType: SQLUSMALLINT,
        rgbDesc: SQLPOINTER,
        cbDescMax: SQLSMALLINT,
        pcbDesc: *mut SQLSMALLINT,
        pfDesc: SQLPOINTER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLConnect(
        hdbc: SQLHDBC,
        szDSN: *mut SQLCHAR,
        cbDSN: SQLSMALLINT,
        szUID: *mut SQLCHAR,
        cbUID: SQLSMALLINT,
        szAuthStr: *mut SQLCHAR,
        cbAuthStr: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLDescribeCol(
        hstmt: SQLHSTMT,
        icol: SQLUSMALLINT,
        szColName: *mut SQLCHAR,
        cbColNameMax: SQLSMALLINT,
        pcbColName: *mut SQLSMALLINT,
        pfSqlType: *mut SQLSMALLINT,
        pcbColDef: *mut SQLUINTEGER,
        pibScale: *mut SQLSMALLINT,
        pfNullable: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLDisconnect(hdbc: SQLHDBC) -> SQLRETURN;
}
extern "C" {
    pub fn SQLError(
        henv: SQLHENV,
        hdbc: SQLHDBC,
        hstmt: SQLHSTMT,
        szSqlState: *mut SQLCHAR,
        pfNativeError: *mut SQLINTEGER,
        szErrorMsg: *mut SQLCHAR,
        cbErrorMsgMax: SQLSMALLINT,
        pcbErrorMsg: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLExecDirect(
        hstmt: SQLHSTMT,
        szSqlStr: *mut SQLCHAR,
        cbSqlStr: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLExecute(hstmt: SQLHSTMT) -> SQLRETURN;
}
extern "C" {
    pub fn SQLFetch(hstmt: SQLHSTMT) -> SQLRETURN;
}
extern "C" {
    pub fn SQLFreeConnect(hdbc: SQLHDBC) -> SQLRETURN;
}
extern "C" {
    pub fn SQLFreeEnv(henv: SQLHENV) -> SQLRETURN;
}
extern "C" {
    pub fn SQLFreeStmt(hstmt: SQLHSTMT, fOption: SQLUSMALLINT) -> SQLRETURN;
}
extern "C" {
    pub fn SQLCloseCursor(hStmt: SQLHSTMT) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetCursorName(
        hstmt: SQLHSTMT,
        szCursor: *mut SQLCHAR,
        cbCursorMax: SQLSMALLINT,
        pcbCursor: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetData(
        hstmt: SQLHSTMT,
        icol: SQLUSMALLINT,
        fCType: SQLSMALLINT,
        rgbValue: SQLPOINTER,
        cbValueMax: SQLINTEGER,
        pcbValue: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLNumResultCols(hstmt: SQLHSTMT, pccol: *mut SQLSMALLINT) -> SQLRETURN;
}
extern "C" {
    pub fn SQLPrepare(hstmt: SQLHSTMT, szSqlStr: *mut SQLCHAR, cbSqlStr: SQLINTEGER) -> SQLRETURN;
}
extern "C" {
    pub fn SQLRowCount(hstmt: SQLHSTMT, pcrow: *mut SQLINTEGER) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetCursorName(
        hstmt: SQLHSTMT,
        szCursor: *mut SQLCHAR,
        cbCursor: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetParam(
        hstmt: SQLHSTMT,
        ipar: SQLUSMALLINT,
        fCType: SQLSMALLINT,
        fSqlType: SQLSMALLINT,
        cbParamDef: SQLUINTEGER,
        ibScale: SQLSMALLINT,
        rgbValue: SQLPOINTER,
        pcbValue: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLTransact(henv: SQLHENV, hdbc: SQLHDBC, fType: SQLUSMALLINT) -> SQLRETURN;
}
extern "C" {
    pub fn SQLEndTran(
        fHandleType: SQLSMALLINT,
        hHandle: SQLHANDLE,
        fType: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLFreeHandle(fHandleType: SQLSMALLINT, hHandle: SQLHANDLE) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetDiagRec(
        fHandleType: SQLSMALLINT,
        hHandle: SQLHANDLE,
        iRecNumber: SQLSMALLINT,
        pszSqlState: *mut SQLCHAR,
        pfNativeError: *mut SQLINTEGER,
        pszErrorMsg: *mut SQLCHAR,
        cbErrorMsgMax: SQLSMALLINT,
        pcbErrorMsg: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetDiagField(
        fHandleType: SQLSMALLINT,
        hHandle: SQLHANDLE,
        iRecNumber: SQLSMALLINT,
        fDiagIdentifier: SQLSMALLINT,
        pDiagInfo: SQLPOINTER,
        cbDiagInfoMax: SQLSMALLINT,
        pcbDiagInfo: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLCopyDesc(hDescSource: SQLHDESC, hDescTarget: SQLHDESC) -> SQLRETURN;
}
extern "C" {
    pub fn SQLCreateDb(
        hDbc: SQLHDBC,
        szDB: *mut SQLCHAR,
        cbDB: SQLINTEGER,
        szCodeset: *mut SQLCHAR,
        cbCodeset: SQLINTEGER,
        szMode: *mut SQLCHAR,
        cbMode: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLDropDb(hDbc: SQLHDBC, szDB: *mut SQLCHAR, cbDB: SQLINTEGER) -> SQLRETURN;
}
extern "C" {
    pub fn SQLCreatePkg(
        hDbc: SQLHDBC,
        szBindFileName: *mut SQLCHAR,
        cbBindFileName: SQLINTEGER,
        szBindOpts: *mut SQLCHAR,
        cbBindOpts: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetDescField(
        DescriptorHandle: SQLHDESC,
        RecNumber: SQLSMALLINT,
        FieldIdentifier: SQLSMALLINT,
        Value: SQLPOINTER,
        BufferLength: SQLINTEGER,
        StringLength: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetDescRec(
        DescriptorHandle: SQLHDESC,
        RecNumber: SQLSMALLINT,
        Name: *mut SQLCHAR,
        BufferLength: SQLSMALLINT,
        StringLength: *mut SQLSMALLINT,
        Type: *mut SQLSMALLINT,
        SubType: *mut SQLSMALLINT,
        Length: *mut SQLINTEGER,
        Precision: *mut SQLSMALLINT,
        Scale: *mut SQLSMALLINT,
        Nullable: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetDescField(
        DescriptorHandle: SQLHDESC,
        RecNumber: SQLSMALLINT,
        FieldIdentifier: SQLSMALLINT,
        Value: SQLPOINTER,
        BufferLength: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetDescRec(
        DescriptorHandle: SQLHDESC,
        RecNumber: SQLSMALLINT,
        Type: SQLSMALLINT,
        SubType: SQLSMALLINT,
        Length: SQLINTEGER,
        Precision: SQLSMALLINT,
        Scale: SQLSMALLINT,
        Data: SQLPOINTER,
        StringLength: *mut SQLINTEGER,
        Indicator: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
pub type LPWSTR = *mut SQLWCHAR;
pub type DWORD = sqluint32;
pub type BOOL = ::std::os::raw::c_uint;
pub type WCHAR = wchar_t;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _TAGGUID {
    pub Data1: ::std::os::raw::c_ulong,
    pub Data2: ::std::os::raw::c_ushort,
    pub Data3: ::std::os::raw::c_ushort,
    pub Data4: [::std::os::raw::c_uchar; 8usize],
}

pub type TAGGUID = _TAGGUID;
pub type SQLSTATE = [SQLTCHAR; 6usize];
extern "C" {
    pub fn SQLDriverConnect(
        hdbc: SQLHDBC,
        hwnd: SQLHWND,
        szConnStrIn: *mut SQLCHAR,
        cchConnStrIn: SQLSMALLINT,
        szConnStrOut: *mut SQLCHAR,
        cchConnStrOutMax: SQLSMALLINT,
        pcchConnStrOut: *mut SQLSMALLINT,
        fDriverCompletion: SQLUSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLBrowseConnect(
        hdbc: SQLHDBC,
        szConnStrIn: *mut SQLCHAR,
        cchConnStrIn: SQLSMALLINT,
        szConnStrOut: *mut SQLCHAR,
        cchConnStrOutMax: SQLSMALLINT,
        pcchConnStrOut: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLBulkOperations(StatementHandle: SQLHSTMT, Operation: SQLSMALLINT) -> SQLRETURN;
}
extern "C" {
    pub fn SQLColAttributes(
        hstmt: SQLHSTMT,
        icol: SQLUSMALLINT,
        fDescType: SQLUSMALLINT,
        rgbDesc: SQLPOINTER,
        cbDescMax: SQLSMALLINT,
        pcbDesc: *mut SQLSMALLINT,
        pfDesc: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLColumnPrivileges(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLCHAR,
        cchCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLCHAR,
        cchSchemaName: SQLSMALLINT,
        szTableName: *mut SQLCHAR,
        cchTableName: SQLSMALLINT,
        szColumnName: *mut SQLCHAR,
        cchColumnName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLDescribeParam(
        hstmt: SQLHSTMT,
        ipar: SQLUSMALLINT,
        pfSqlType: *mut SQLSMALLINT,
        pcbParamDef: *mut SQLUINTEGER,
        pibScale: *mut SQLSMALLINT,
        pfNullable: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLExtendedFetch(
        hstmt: SQLHSTMT,
        fFetchType: SQLUSMALLINT,
        irow: SQLINTEGER,
        pcrow: *mut SQLUINTEGER,
        rgfRowStatus: *mut SQLUSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLForeignKeys(
        hstmt: SQLHSTMT,
        szPkCatalogName: *mut SQLCHAR,
        cchPkCatalogName: SQLSMALLINT,
        szPkSchemaName: *mut SQLCHAR,
        cchPkSchemaName: SQLSMALLINT,
        szPkTableName: *mut SQLCHAR,
        cchPkTableName: SQLSMALLINT,
        szFkCatalogName: *mut SQLCHAR,
        cchFkCatalogName: SQLSMALLINT,
        szFkSchemaName: *mut SQLCHAR,
        cchFkSchemaName: SQLSMALLINT,
        szFkTableName: *mut SQLCHAR,
        cchFkTableName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLMoreResults(hstmt: SQLHSTMT) -> SQLRETURN;
}
extern "C" {
    pub fn SQLNativeSql(
        hdbc: SQLHDBC,
        szSqlStrIn: *mut SQLCHAR,
        cchSqlStrIn: SQLINTEGER,
        szSqlStr: *mut SQLCHAR,
        cchSqlStrMax: SQLINTEGER,
        pcbSqlStr: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLNumParams(hstmt: SQLHSTMT, pcpar: *mut SQLSMALLINT) -> SQLRETURN;
}
extern "C" {
    pub fn SQLParamOptions(
        hstmt: SQLHSTMT,
        crow: SQLUINTEGER,
        pirow: *mut SQLUINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLPrimaryKeys(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLCHAR,
        cchCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLCHAR,
        cchSchemaName: SQLSMALLINT,
        szTableName: *mut SQLCHAR,
        cchTableName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLProcedureColumns(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLCHAR,
        cchCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLCHAR,
        cchSchemaName: SQLSMALLINT,
        szProcName: *mut SQLCHAR,
        cchProcName: SQLSMALLINT,
        szColumnName: *mut SQLCHAR,
        cchColumnName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLProcedures(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLCHAR,
        cchCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLCHAR,
        cchSchemaName: SQLSMALLINT,
        szProcName: *mut SQLCHAR,
        cchProcName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetPos(
        hstmt: SQLHSTMT,
        irow: SQLUSMALLINT,
        fOption: SQLUSMALLINT,
        fLock: SQLUSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLTablePrivileges(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLCHAR,
        cchCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLCHAR,
        cchSchemaName: SQLSMALLINT,
        szTableName: *mut SQLCHAR,
        cchTableName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLDrivers(
        henv: SQLHENV,
        fDirection: SQLUSMALLINT,
        szDriverDesc: *mut SQLCHAR,
        cchDriverDescMax: SQLSMALLINT,
        pcchDriverDesc: *mut SQLSMALLINT,
        szDriverAttributes: *mut SQLCHAR,
        cchDrvrAttrMax: SQLSMALLINT,
        pcchDrvrAttr: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLBindParameter(
        hstmt: SQLHSTMT,
        ipar: SQLUSMALLINT,
        fParamType: SQLSMALLINT,
        fCType: SQLSMALLINT,
        fSqlType: SQLSMALLINT,
        cbColDef: SQLUINTEGER,
        ibScale: SQLSMALLINT,
        rgbValue: SQLPOINTER,
        cbValueMax: SQLINTEGER,
        pcbValue: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLAllocHandleStd(
        fHandleType: SQLSMALLINT,
        hInput: SQLHANDLE,
        phOutput: *mut SQLHANDLE,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetScrollOptions(
        hstmt: SQLHSTMT,
        fConcurrency: SQLUSMALLINT,
        crowKeyset: SQLINTEGER,
        crowRowset: SQLUSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn TraceOpenLogFile(
        szFileName: LPWSTR,
        lpwszOutputMsg: LPWSTR,
        cbOutputMsg: DWORD,
    ) -> RETCODE;
}
extern "C" {
    pub fn TraceCloseLogFile() -> RETCODE;
}
extern "C" {
    pub fn TraceReturn(arg1: RETCODE, arg2: RETCODE);
}
extern "C" {
    pub fn TraceVersion() -> DWORD;
}
extern "C" {
    pub fn TraceVSControl(arg1: DWORD) -> RETCODE;
}
extern "C" {
    pub fn ODBCSetTryWaitValue(dwValue: DWORD) -> BOOL;
}
extern "C" {
    pub fn ODBCGetTryWaitValue() -> DWORD;
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct tagODBC_VS_ARGS {
    pub pguidEvent: *const TAGGUID,
    pub dwFlags: DWORD,
    pub __bindgen_anon_1: tagODBC_VS_ARGS__bindgen_ty_1,
    pub __bindgen_anon_2: tagODBC_VS_ARGS__bindgen_ty_2,
    pub RetCode: RETCODE,
}
#[repr(C)]
#[derive(Copy, Clone)]
pub union tagODBC_VS_ARGS__bindgen_ty_1 {
    pub wszArg: *mut WCHAR,
    pub szArg: *mut ::std::os::raw::c_char,
    _bindgen_union_align: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union tagODBC_VS_ARGS__bindgen_ty_2 {
    pub wszCorrelation: *mut WCHAR,
    pub szCorrelation: *mut ::std::os::raw::c_char,
    _bindgen_union_align: u64,
}

pub type ODBC_VS_ARGS = tagODBC_VS_ARGS;
pub type PODBC_VS_ARGS = *mut tagODBC_VS_ARGS;
extern "C" {
    pub fn FireVSDebugEvent(arg1: PODBC_VS_ARGS);
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SQL_NET_STATS {
    pub iNetStatsLength: SQLINTEGER,
    pub uiNetStatsServerTime: SQLUBIGINT,
    pub uiNetStatsNetworkTime: SQLUBIGINT,
    pub uiNetStatsBytesSent: SQLUBIGINT,
    pub uiNetStatsBytesReceived: SQLUBIGINT,
    pub uiNetStatsRoundTrips: SQLUBIGINT,
}

extern "C" {
    pub fn SQLColumns(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLCHAR,
        cbSchemaName: SQLSMALLINT,
        szTableName: *mut SQLCHAR,
        cbTableName: SQLSMALLINT,
        szColumnName: *mut SQLCHAR,
        cbColumnName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLDataSources(
        henv: SQLHENV,
        fDirection: SQLUSMALLINT,
        szDSN: *mut SQLCHAR,
        cbDSNMax: SQLSMALLINT,
        pcbDSN: *mut SQLSMALLINT,
        szDescription: *mut SQLCHAR,
        cbDescriptionMax: SQLSMALLINT,
        pcbDescription: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLFetchScroll(
        StatementHandle: SQLHSTMT,
        FetchOrientation: SQLSMALLINT,
        FetchOffset: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetConnectAttr(
        ConnectionHandle: SQLHDBC,
        Attribute: SQLINTEGER,
        Value: SQLPOINTER,
        BufferLength: SQLINTEGER,
        StringLength: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetConnectOption(
        hdbc: SQLHDBC,
        fOption: SQLUSMALLINT,
        pvParam: SQLPOINTER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetFunctions(
        hdbc: SQLHDBC,
        fFunction: SQLUSMALLINT,
        pfExists: *mut SQLUSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetInfo(
        hdbc: SQLHDBC,
        fInfoType: SQLUSMALLINT,
        rgbInfoValue: SQLPOINTER,
        cbInfoValueMax: SQLSMALLINT,
        pcbInfoValue: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetStmtAttr(
        StatementHandle: SQLHSTMT,
        Attribute: SQLINTEGER,
        Value: SQLPOINTER,
        BufferLength: SQLINTEGER,
        StringLength: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetStmtOption(
        hstmt: SQLHSTMT,
        fOption: SQLUSMALLINT,
        pvParam: SQLPOINTER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetTypeInfo(hstmt: SQLHSTMT, fSqlType: SQLSMALLINT) -> SQLRETURN;
}
extern "C" {
    pub fn SQLParamData(hstmt: SQLHSTMT, prgbValue: *mut SQLPOINTER) -> SQLRETURN;
}
extern "C" {
    pub fn SQLPutData(hstmt: SQLHSTMT, rgbValue: SQLPOINTER, cbValue: SQLINTEGER) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetConnectAttr(
        hdbc: SQLHDBC,
        fOption: SQLINTEGER,
        pvParam: SQLPOINTER,
        fStrLen: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetConnectOption(
        hdbc: SQLHDBC,
        fOption: SQLUSMALLINT,
        vParam: SQLUINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetStmtAttr(
        hstmt: SQLHSTMT,
        fOption: SQLINTEGER,
        pvParam: SQLPOINTER,
        fStrLen: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetStmtOption(
        hstmt: SQLHSTMT,
        fOption: SQLUSMALLINT,
        vParam: SQLUINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSpecialColumns(
        hstmt: SQLHSTMT,
        fColType: SQLUSMALLINT,
        szCatalogName: *mut SQLCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLCHAR,
        cbSchemaName: SQLSMALLINT,
        szTableName: *mut SQLCHAR,
        cbTableName: SQLSMALLINT,
        fScope: SQLUSMALLINT,
        fNullable: SQLUSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLStatistics(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLCHAR,
        cbSchemaName: SQLSMALLINT,
        szTableName: *mut SQLCHAR,
        cbTableName: SQLSMALLINT,
        fUnique: SQLUSMALLINT,
        fAccuracy: SQLUSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLTables(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLCHAR,
        cbSchemaName: SQLSMALLINT,
        szTableName: *mut SQLCHAR,
        cbTableName: SQLSMALLINT,
        szTableType: *mut SQLCHAR,
        cbTableType: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLNextResult(hstmtSource: SQLHSTMT, hstmtTarget: SQLHSTMT) -> SQLRETURN;
}
extern "C" {
    pub fn SQLColAttributeW(
        hstmt: SQLHSTMT,
        iCol: SQLUSMALLINT,
        iField: SQLUSMALLINT,
        pCharAttr: SQLPOINTER,
        cbCharAttrMax: SQLSMALLINT,
        pcbCharAttr: *mut SQLSMALLINT,
        pNumAttr: SQLPOINTER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLColAttributesW(
        hstmt: SQLHSTMT,
        icol: SQLUSMALLINT,
        fDescType: SQLUSMALLINT,
        rgbDesc: SQLPOINTER,
        cbDescMax: SQLSMALLINT,
        pcbDesc: *mut SQLSMALLINT,
        pfDesc: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLConnectW(
        hdbc: SQLHDBC,
        szDSN: *mut SQLWCHAR,
        cbDSN: SQLSMALLINT,
        szUID: *mut SQLWCHAR,
        cbUID: SQLSMALLINT,
        szAuthStr: *mut SQLWCHAR,
        cbAuthStr: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLConnectWInt(
        hdbc: SQLHDBC,
        szDSN: *mut SQLWCHAR,
        cbDSN: SQLSMALLINT,
        szUID: *mut SQLWCHAR,
        cbUID: SQLSMALLINT,
        szAuthStr: *mut SQLWCHAR,
        cbAuthStr: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLDescribeColW(
        hstmt: SQLHSTMT,
        icol: SQLUSMALLINT,
        szColName: *mut SQLWCHAR,
        cbColNameMax: SQLSMALLINT,
        pcbColName: *mut SQLSMALLINT,
        pfSqlType: *mut SQLSMALLINT,
        pcbColDef: *mut SQLUINTEGER,
        pibScale: *mut SQLSMALLINT,
        pfNullable: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLErrorW(
        henv: SQLHENV,
        hdbc: SQLHDBC,
        hstmt: SQLHSTMT,
        szSqlState: *mut SQLWCHAR,
        pfNativeError: *mut SQLINTEGER,
        szErrorMsg: *mut SQLWCHAR,
        cbErrorMsgMax: SQLSMALLINT,
        pcbErrorMsg: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLExecDirectW(
        hstmt: SQLHSTMT,
        szSqlStr: *mut SQLWCHAR,
        cbSqlStr: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetConnectAttrW(
        hdbc: SQLHDBC,
        fAttribute: SQLINTEGER,
        rgbValue: SQLPOINTER,
        cbValueMax: SQLINTEGER,
        pcbValue: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetCursorNameW(
        hstmt: SQLHSTMT,
        szCursor: *mut SQLWCHAR,
        cbCursorMax: SQLSMALLINT,
        pcbCursor: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetDescFieldW(
        DescriptorHandle: SQLHDESC,
        RecNumber: SQLSMALLINT,
        FieldIdentifier: SQLSMALLINT,
        Value: SQLPOINTER,
        BufferLength: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetDescFieldW(
        hdesc: SQLHDESC,
        iRecord: SQLSMALLINT,
        iField: SQLSMALLINT,
        rgbValue: SQLPOINTER,
        cbValueMax: SQLINTEGER,
        pcbValue: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetDescRecW(
        hdesc: SQLHDESC,
        iRecord: SQLSMALLINT,
        szName: *mut SQLWCHAR,
        cbNameMax: SQLSMALLINT,
        pcbName: *mut SQLSMALLINT,
        pfType: *mut SQLSMALLINT,
        pfSubType: *mut SQLSMALLINT,
        pLength: *mut SQLINTEGER,
        pPrecision: *mut SQLSMALLINT,
        pScale: *mut SQLSMALLINT,
        pNullable: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetDiagFieldW(
        fHandleType: SQLSMALLINT,
        handle: SQLHANDLE,
        iRecord: SQLSMALLINT,
        fDiagField: SQLSMALLINT,
        rgbDiagInfo: SQLPOINTER,
        cbDiagInfoMax: SQLSMALLINT,
        pcbDiagInfo: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetDiagRecW(
        fHandleType: SQLSMALLINT,
        handle: SQLHANDLE,
        iRecord: SQLSMALLINT,
        szSqlState: *mut SQLWCHAR,
        pfNativeError: *mut SQLINTEGER,
        szErrorMsg: *mut SQLWCHAR,
        cbErrorMsgMax: SQLSMALLINT,
        pcbErrorMsg: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetEnvAttrW(
        hEnv: SQLHENV,
        fAttribute: SQLINTEGER,
        pParam: SQLPOINTER,
        cbParamMax: SQLINTEGER,
        pcbParam: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLPrepareW(hstmt: SQLHSTMT, szSqlStr: *mut SQLWCHAR, cbSqlStr: SQLINTEGER)
        -> SQLRETURN;
}
extern "C" {
    pub fn SQLExtendedPrepareW(
        hStmt: SQLHSTMT,
        pszSqlStrIn: *mut SQLWCHAR,
        cbSqlStr: SQLINTEGER,
        cPars: SQLINTEGER,
        sStmtType: SQLSMALLINT,
        cStmtAttrs: SQLINTEGER,
        piStmtAttr: *mut SQLINTEGER,
        pvParams: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetConnectAttrW(
        hdbc: SQLHDBC,
        fAttribute: SQLINTEGER,
        rgbValue: SQLPOINTER,
        cbValue: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetCursorNameW(
        hstmt: SQLHSTMT,
        szCursor: *mut SQLWCHAR,
        cbCursor: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetEnvAttrW(
        hEnv: SQLHENV,
        fAttribute: SQLINTEGER,
        pParam: SQLPOINTER,
        cbParam: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLColumnsW(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLWCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLWCHAR,
        cbSchemaName: SQLSMALLINT,
        szTableName: *mut SQLWCHAR,
        cbTableName: SQLSMALLINT,
        szColumnName: *mut SQLWCHAR,
        cbColumnName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetInfoW(
        hdbc: SQLHDBC,
        fInfoType: SQLUSMALLINT,
        rgbInfoValue: SQLPOINTER,
        cbInfoValueMax: SQLSMALLINT,
        pcbInfoValue: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetConnectOptionW(
        hDbc: SQLHDBC,
        fOptionIn: SQLUSMALLINT,
        pvParam: SQLPOINTER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetConnectOptionW(
        hDbc: SQLHDBC,
        fOptionIn: SQLUSMALLINT,
        vParam: SQLUINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetTypeInfoW(hstmt: SQLHSTMT, fSqlType: SQLSMALLINT) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSpecialColumnsW(
        hstmt: SQLHSTMT,
        fColType: SQLUSMALLINT,
        szCatalogName: *mut SQLWCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLWCHAR,
        cbSchemaName: SQLSMALLINT,
        szTableName: *mut SQLWCHAR,
        cbTableName: SQLSMALLINT,
        fScope: SQLUSMALLINT,
        fNullable: SQLUSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLStatisticsW(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLWCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLWCHAR,
        cbSchemaName: SQLSMALLINT,
        szTableName: *mut SQLWCHAR,
        cbTableName: SQLSMALLINT,
        fUnique: SQLUSMALLINT,
        fAccuracy: SQLUSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLTablesW(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLWCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLWCHAR,
        cbSchemaName: SQLSMALLINT,
        szTableName: *mut SQLWCHAR,
        cbTableName: SQLSMALLINT,
        szTableType: *mut SQLWCHAR,
        cbTableType: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLDataSourcesW(
        henv: SQLHENV,
        fDirection: SQLUSMALLINT,
        szDSN: *mut SQLWCHAR,
        cbDSNMax: SQLSMALLINT,
        pcbDSN: *mut SQLSMALLINT,
        szDescription: *mut SQLWCHAR,
        cbDescriptionMax: SQLSMALLINT,
        pcbDescription: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLDriverConnectW(
        hdbc: SQLHDBC,
        hwnd: SQLHWND,
        szConnStrIn: *mut SQLWCHAR,
        cbConnStrIn: SQLSMALLINT,
        szConnStrOut: *mut SQLWCHAR,
        cbConnStrOutMax: SQLSMALLINT,
        pcbConnStrOut: *mut SQLSMALLINT,
        fDriverCompletion: SQLUSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLBrowseConnectW(
        hdbc: SQLHDBC,
        szConnStrIn: *mut SQLWCHAR,
        cbConnStrIn: SQLSMALLINT,
        szConnStrOut: *mut SQLWCHAR,
        cbConnStrOutMax: SQLSMALLINT,
        pcbConnStrOut: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLColumnPrivilegesW(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLWCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLWCHAR,
        cbSchemaName: SQLSMALLINT,
        szTableName: *mut SQLWCHAR,
        cbTableName: SQLSMALLINT,
        szColumnName: *mut SQLWCHAR,
        cbColumnName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetStmtAttrW(
        hstmt: SQLHSTMT,
        fAttribute: SQLINTEGER,
        rgbValue: SQLPOINTER,
        cbValueMax: SQLINTEGER,
        pcbValue: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetStmtAttrW(
        hstmt: SQLHSTMT,
        fAttribute: SQLINTEGER,
        rgbValue: SQLPOINTER,
        cbValueMax: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLForeignKeysW(
        hstmt: SQLHSTMT,
        szPkCatalogName: *mut SQLWCHAR,
        cbPkCatalogName: SQLSMALLINT,
        szPkSchemaName: *mut SQLWCHAR,
        cbPkSchemaName: SQLSMALLINT,
        szPkTableName: *mut SQLWCHAR,
        cbPkTableName: SQLSMALLINT,
        szFkCatalogName: *mut SQLWCHAR,
        cbFkCatalogName: SQLSMALLINT,
        szFkSchemaName: *mut SQLWCHAR,
        cbFkSchemaName: SQLSMALLINT,
        szFkTableName: *mut SQLWCHAR,
        cbFkTableName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLNativeSqlW(
        hdbc: SQLHDBC,
        szSqlStrIn: *mut SQLWCHAR,
        cbSqlStrIn: SQLINTEGER,
        szSqlStr: *mut SQLWCHAR,
        cbSqlStrMax: SQLINTEGER,
        pcbSqlStr: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLPrimaryKeysW(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLWCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLWCHAR,
        cbSchemaName: SQLSMALLINT,
        szTableName: *mut SQLWCHAR,
        cbTableName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLProcedureColumnsW(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLWCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLWCHAR,
        cbSchemaName: SQLSMALLINT,
        szProcName: *mut SQLWCHAR,
        cbProcName: SQLSMALLINT,
        szColumnName: *mut SQLWCHAR,
        cbColumnName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLProceduresW(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLWCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLWCHAR,
        cbSchemaName: SQLSMALLINT,
        szProcName: *mut SQLWCHAR,
        cbProcName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLExtendedProcedureColumnsW(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLWCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLWCHAR,
        cbSchemaName: SQLSMALLINT,
        szProcName: *mut SQLWCHAR,
        cbProcName: SQLSMALLINT,
        szColumnName: *mut SQLWCHAR,
        cbColumnName: SQLSMALLINT,
        szModuleName: *mut SQLWCHAR,
        cbModuleName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLExtendedProceduresW(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLWCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLWCHAR,
        cbSchemaName: SQLSMALLINT,
        szProcName: *mut SQLWCHAR,
        cbProcName: SQLSMALLINT,
        szModuleName: *mut SQLWCHAR,
        cbModuleName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLTablePrivilegesW(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLWCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLWCHAR,
        cbSchemaName: SQLSMALLINT,
        szTableName: *mut SQLWCHAR,
        cbTableName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLCreateDbW(
        hDbc: SQLHDBC,
        pszDBW: *mut SQLWCHAR,
        cbDB: SQLINTEGER,
        pszCodeSetW: *mut SQLWCHAR,
        cbCodeSet: SQLINTEGER,
        pszModeW: *mut SQLWCHAR,
        cbMode: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLDropDbW(hDbc: SQLHDBC, pszDBW: *mut SQLWCHAR, cbDB: SQLINTEGER) -> SQLRETURN;
}
extern "C" {
    pub fn SQLCreatePkgW(
        hDbc: SQLHDBC,
        szBindFileNameIn: *mut SQLWCHAR,
        cbBindFileNameIn: SQLINTEGER,
        szBindOpts: *mut SQLWCHAR,
        cbBindOpts: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLDropPkgW(
        hDbc: SQLHDBC,
        szCollection: *mut SQLWCHAR,
        cbCollection: SQLINTEGER,
        szPackage: *mut SQLWCHAR,
        cbPackage: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLBindFileToCol(
        hstmt: SQLHSTMT,
        icol: SQLUSMALLINT,
        FileName: *mut SQLCHAR,
        FileNameLength: *mut SQLSMALLINT,
        FileOptions: *mut SQLUINTEGER,
        MaxFileNameLength: SQLSMALLINT,
        StringLength: *mut SQLINTEGER,
        IndicatorValue: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLBindFileToParam(
        hstmt: SQLHSTMT,
        ipar: SQLUSMALLINT,
        fSqlType: SQLSMALLINT,
        FileName: *mut SQLCHAR,
        FileNameLength: *mut SQLSMALLINT,
        FileOptions: *mut SQLUINTEGER,
        MaxFileNameLength: SQLSMALLINT,
        IndicatorValue: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetLength(
        hstmt: SQLHSTMT,
        LocatorCType: SQLSMALLINT,
        Locator: SQLINTEGER,
        StringLength: *mut SQLINTEGER,
        IndicatorValue: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetPosition(
        hstmt: SQLHSTMT,
        LocatorCType: SQLSMALLINT,
        SourceLocator: SQLINTEGER,
        SearchLocator: SQLINTEGER,
        SearchLiteral: *mut SQLCHAR,
        SearchLiteralLength: SQLINTEGER,
        FromPosition: SQLUINTEGER,
        LocatedAt: *mut SQLUINTEGER,
        IndicatorValue: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetSQLCA(
        henv: SQLHENV,
        hdbc: SQLHDBC,
        hstmt: SQLHSTMT,
        pSqlca: *mut sqlca,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetSubString(
        hstmt: SQLHSTMT,
        LocatorCType: SQLSMALLINT,
        SourceLocator: SQLINTEGER,
        FromPosition: SQLUINTEGER,
        ForLength: SQLUINTEGER,
        TargetCType: SQLSMALLINT,
        rgbValue: SQLPOINTER,
        cbValueMax: SQLINTEGER,
        StringLength: *mut SQLINTEGER,
        IndicatorValue: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetColAttributes(
        hstmt: SQLHSTMT,
        icol: SQLUSMALLINT,
        pszColName: *mut SQLCHAR,
        cbColName: SQLSMALLINT,
        fSQLType: SQLSMALLINT,
        cbColDef: SQLUINTEGER,
        ibScale: SQLSMALLINT,
        fNullable: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLExtendedProcedures(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLCHAR,
        cbSchemaName: SQLSMALLINT,
        szProcName: *mut SQLCHAR,
        cbProcName: SQLSMALLINT,
        szModuleName: *mut SQLCHAR,
        cbModuleName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLExtendedProcedureColumns(
        hstmt: SQLHSTMT,
        szCatalogName: *mut SQLCHAR,
        cbCatalogName: SQLSMALLINT,
        szSchemaName: *mut SQLCHAR,
        cbSchemaName: SQLSMALLINT,
        szProcName: *mut SQLCHAR,
        cbProcName: SQLSMALLINT,
        szColumnName: *mut SQLCHAR,
        cbColumnName: SQLSMALLINT,
        szModuleName: *mut SQLCHAR,
        cbModuleName: SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLReloadConfig(
        config_property: SQLINTEGER,
        DiagInfoString: *mut SQLCHAR,
        BufferLength: SQLSMALLINT,
        StringLengthPtr: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLReloadConfigW(
        config_property: SQLINTEGER,
        DiagInfoString: *mut SQLWCHAR,
        BufferLength: SQLSMALLINT,
        StringLengthPtr: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetPositionW(
        hStmt: SQLHSTMT,
        fCType: SQLSMALLINT,
        iLocatorIn: SQLINTEGER,
        iPatternLocator: SQLINTEGER,
        pszPatternLiteral: *mut SQLWCHAR,
        cbPatternLiteral: SQLINTEGER,
        iStartSearchAtIn: SQLUINTEGER,
        piLocatedAtIn: *mut SQLUINTEGER,
        piIndicatorValue: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetConnection(hdbc: SQLHDBC) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetEnvAttr(
        henv: SQLHENV,
        Attribute: SQLINTEGER,
        Value: SQLPOINTER,
        BufferLength: SQLINTEGER,
        StringLength: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLSetEnvAttr(
        henv: SQLHENV,
        Attribute: SQLINTEGER,
        Value: SQLPOINTER,
        StringLength: SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLBindParam(
        StatementHandle: SQLHSTMT,
        ParameterNumber: SQLUSMALLINT,
        ValueType: SQLSMALLINT,
        ParameterType: SQLSMALLINT,
        LengthPrecision: SQLUINTEGER,
        ParameterScale: SQLSMALLINT,
        ParameterValue: SQLPOINTER,
        StrLen_or_Ind: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLBuildDataLink(
        hStmt: SQLHSTMT,
        pszLinkType: *mut SQLCHAR,
        cbLinkType: SQLINTEGER,
        pszDataLocation: *mut SQLCHAR,
        cbDataLocation: SQLINTEGER,
        pszComment: *mut SQLCHAR,
        cbComment: SQLINTEGER,
        pDataLink: *mut SQLCHAR,
        cbDataLinkMax: SQLINTEGER,
        pcbDataLink: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLGetDataLinkAttr(
        hStmt: SQLHSTMT,
        fAttrType: SQLSMALLINT,
        pDataLink: *mut SQLCHAR,
        cbDataLink: SQLINTEGER,
        pAttribute: SQLPOINTER,
        cbAttributeMax: SQLINTEGER,
        pcbAttribute: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLExtendedPrepare(
        hstmt: SQLHSTMT,
        pszSqlStmt: *mut SQLCHAR,
        cbSqlStmt: SQLINTEGER,
        cPars: SQLINTEGER,
        sStmtType: SQLSMALLINT,
        cStmtAttrs: SQLINTEGER,
        piStmtAttr: *mut SQLINTEGER,
        pvParams: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLExtendedBind(
        hstmt: SQLHSTMT,
        fBindCol: SQLSMALLINT,
        cRecords: SQLSMALLINT,
        pfCType: *mut SQLSMALLINT,
        rgbValue: *mut SQLPOINTER,
        cbValueMax: *mut SQLINTEGER,
        puiPrecisionCType: *mut SQLUINTEGER,
        psScaleCType: *mut SQLSMALLINT,
        pcbValue: *mut *mut SQLINTEGER,
        piIndicatorPtr: *mut *mut SQLINTEGER,
        pfParamType: *mut SQLSMALLINT,
        pfSQLType: *mut SQLSMALLINT,
        pcbColDef: *mut SQLUINTEGER,
        pibScale: *mut SQLSMALLINT,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLExtendedDescribe(
        hStmt: SQLHANDLE,
        fDescribeCol: SQLSMALLINT,
        iNumRecordsAllocated: SQLUSMALLINT,
        pusNumRecords: *mut SQLUSMALLINT,
        pNames: *mut SQLCHAR,
        sNameMaxByteLen: SQLSMALLINT,
        psNameCharLen: *mut SQLSMALLINT,
        psSQLType: *mut SQLSMALLINT,
        pcbColDef: *mut SQLUINTEGER,
        pcbDisplaySize: *mut SQLUINTEGER,
        psScale: *mut SQLSMALLINT,
        psNullable: *mut SQLSMALLINT,
        psParamType: *mut SQLSMALLINT,
        piCardinality: *mut SQLINTEGER,
    ) -> SQLRETURN;
}
extern "C" {
    pub fn SQLDropPkg(
        hDbc: SQLHDBC,
        szCollection: *mut SQLCHAR,
        cbCollection: SQLINTEGER,
        szPackage: *mut SQLCHAR,
        cbPackage: SQLINTEGER,
    ) -> SQLRETURN;
}

#[cfg(test)]
#[allow(deref_nullptr)]
mod tests {
    use super::*;

    #[test]
    fn bindgen_test_layout_sqlca() {
        assert_eq!(
            ::std::mem::size_of::<sqlca>(),
            136usize,
            concat!("Size of: ", stringify!(sqlca))
        );
        assert_eq!(
            ::std::mem::align_of::<sqlca>(),
            4usize,
            concat!("Alignment of ", stringify!(sqlca))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<sqlca>())).sqlcaid as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(sqlca),
                "::",
                stringify!(sqlcaid)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<sqlca>())).sqlcabc as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(sqlca),
                "::",
                stringify!(sqlcabc)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<sqlca>())).sqlcode as *const _ as usize },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(sqlca),
                "::",
                stringify!(sqlcode)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<sqlca>())).sqlerrml as *const _ as usize },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(sqlca),
                "::",
                stringify!(sqlerrml)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<sqlca>())).sqlerrmc as *const _ as usize },
            18usize,
            concat!(
                "Offset of field: ",
                stringify!(sqlca),
                "::",
                stringify!(sqlerrmc)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<sqlca>())).sqlerrp as *const _ as usize },
            88usize,
            concat!(
                "Offset of field: ",
                stringify!(sqlca),
                "::",
                stringify!(sqlerrp)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<sqlca>())).sqlerrd as *const _ as usize },
            96usize,
            concat!(
                "Offset of field: ",
                stringify!(sqlca),
                "::",
                stringify!(sqlerrd)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<sqlca>())).sqlwarn as *const _ as usize },
            120usize,
            concat!(
                "Offset of field: ",
                stringify!(sqlca),
                "::",
                stringify!(sqlwarn)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<sqlca>())).sqlstate as *const _ as usize },
            131usize,
            concat!(
                "Offset of field: ",
                stringify!(sqlca),
                "::",
                stringify!(sqlstate)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_div_t() {
        assert_eq!(
            ::std::mem::size_of::<div_t>(),
            8usize,
            concat!("Size of: ", stringify!(div_t))
        );
        assert_eq!(
            ::std::mem::align_of::<div_t>(),
            4usize,
            concat!("Alignment of ", stringify!(div_t))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<div_t>())).quot as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(div_t),
                "::",
                stringify!(quot)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<div_t>())).rem as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(div_t),
                "::",
                stringify!(rem)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_ldiv_t() {
        assert_eq!(
            ::std::mem::size_of::<ldiv_t>(),
            16usize,
            concat!("Size of: ", stringify!(ldiv_t))
        );
        assert_eq!(
            ::std::mem::align_of::<ldiv_t>(),
            8usize,
            concat!("Alignment of ", stringify!(ldiv_t))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ldiv_t>())).quot as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(ldiv_t),
                "::",
                stringify!(quot)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<ldiv_t>())).rem as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(ldiv_t),
                "::",
                stringify!(rem)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_lldiv_t() {
        assert_eq!(
            ::std::mem::size_of::<lldiv_t>(),
            16usize,
            concat!("Size of: ", stringify!(lldiv_t))
        );
        assert_eq!(
            ::std::mem::align_of::<lldiv_t>(),
            8usize,
            concat!("Alignment of ", stringify!(lldiv_t))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<lldiv_t>())).quot as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(lldiv_t),
                "::",
                stringify!(quot)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<lldiv_t>())).rem as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(lldiv_t),
                "::",
                stringify!(rem)
            )
        );
    }

    #[test]
    fn bindgen_test_layout___fsid_t() {
        assert_eq!(
            ::std::mem::size_of::<__fsid_t>(),
            8usize,
            concat!("Size of: ", stringify!(__fsid_t))
        );
        assert_eq!(
            ::std::mem::align_of::<__fsid_t>(),
            4usize,
            concat!("Alignment of ", stringify!(__fsid_t))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<__fsid_t>())).__val as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(__fsid_t),
                "::",
                stringify!(__val)
            )
        );
    }

    #[test]
    fn bindgen_test_layout___sigset_t() {
        assert_eq!(
            ::std::mem::size_of::<__sigset_t>(),
            128usize,
            concat!("Size of: ", stringify!(__sigset_t))
        );
        assert_eq!(
            ::std::mem::align_of::<__sigset_t>(),
            8usize,
            concat!("Alignment of ", stringify!(__sigset_t))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<__sigset_t>())).__val as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(__sigset_t),
                "::",
                stringify!(__val)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_timeval() {
        assert_eq!(
            ::std::mem::size_of::<timeval>(),
            16usize,
            concat!("Size of: ", stringify!(timeval))
        );
        assert_eq!(
            ::std::mem::align_of::<timeval>(),
            8usize,
            concat!("Alignment of ", stringify!(timeval))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<timeval>())).tv_sec as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(timeval),
                "::",
                stringify!(tv_sec)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<timeval>())).tv_usec as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(timeval),
                "::",
                stringify!(tv_usec)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_timespec() {
        assert_eq!(
            ::std::mem::size_of::<timespec>(),
            16usize,
            concat!("Size of: ", stringify!(timespec))
        );
        assert_eq!(
            ::std::mem::align_of::<timespec>(),
            8usize,
            concat!("Alignment of ", stringify!(timespec))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<timespec>())).tv_sec as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(timespec),
                "::",
                stringify!(tv_sec)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<timespec>())).tv_nsec as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(timespec),
                "::",
                stringify!(tv_nsec)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_fd_set() {
        assert_eq!(
            ::std::mem::size_of::<fd_set>(),
            128usize,
            concat!("Size of: ", stringify!(fd_set))
        );
        assert_eq!(
            ::std::mem::align_of::<fd_set>(),
            8usize,
            concat!("Alignment of ", stringify!(fd_set))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<fd_set>())).__fds_bits as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(fd_set),
                "::",
                stringify!(__fds_bits)
            )
        );
    }

    #[test]
    fn bindgen_test_layout___pthread_rwlock_arch_t() {
        assert_eq!(
            ::std::mem::size_of::<__pthread_rwlock_arch_t>(),
            56usize,
            concat!("Size of: ", stringify!(__pthread_rwlock_arch_t))
        );
        assert_eq!(
            ::std::mem::align_of::<__pthread_rwlock_arch_t>(),
            8usize,
            concat!("Alignment of ", stringify!(__pthread_rwlock_arch_t))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_rwlock_arch_t>())).__readers as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_rwlock_arch_t),
                "::",
                stringify!(__readers)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_rwlock_arch_t>())).__writers as *const _ as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_rwlock_arch_t),
                "::",
                stringify!(__writers)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_rwlock_arch_t>())).__wrphase_futex as *const _
                    as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_rwlock_arch_t),
                "::",
                stringify!(__wrphase_futex)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_rwlock_arch_t>())).__writers_futex as *const _
                    as usize
            },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_rwlock_arch_t),
                "::",
                stringify!(__writers_futex)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_rwlock_arch_t>())).__pad3 as *const _ as usize
            },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_rwlock_arch_t),
                "::",
                stringify!(__pad3)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_rwlock_arch_t>())).__pad4 as *const _ as usize
            },
            20usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_rwlock_arch_t),
                "::",
                stringify!(__pad4)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_rwlock_arch_t>())).__cur_writer as *const _
                    as usize
            },
            24usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_rwlock_arch_t),
                "::",
                stringify!(__cur_writer)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_rwlock_arch_t>())).__shared as *const _ as usize
            },
            28usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_rwlock_arch_t),
                "::",
                stringify!(__shared)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_rwlock_arch_t>())).__rwelision as *const _ as usize
            },
            32usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_rwlock_arch_t),
                "::",
                stringify!(__rwelision)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_rwlock_arch_t>())).__pad1 as *const _ as usize
            },
            33usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_rwlock_arch_t),
                "::",
                stringify!(__pad1)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_rwlock_arch_t>())).__pad2 as *const _ as usize
            },
            40usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_rwlock_arch_t),
                "::",
                stringify!(__pad2)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_rwlock_arch_t>())).__flags as *const _ as usize
            },
            48usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_rwlock_arch_t),
                "::",
                stringify!(__flags)
            )
        );
    }

    #[test]
    fn bindgen_test_layout___pthread_internal_list() {
        assert_eq!(
            ::std::mem::size_of::<__pthread_internal_list>(),
            16usize,
            concat!("Size of: ", stringify!(__pthread_internal_list))
        );
        assert_eq!(
            ::std::mem::align_of::<__pthread_internal_list>(),
            8usize,
            concat!("Alignment of ", stringify!(__pthread_internal_list))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_internal_list>())).__prev as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_internal_list),
                "::",
                stringify!(__prev)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_internal_list>())).__next as *const _ as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_internal_list),
                "::",
                stringify!(__next)
            )
        );
    }

    #[test]
    fn bindgen_test_layout___pthread_mutex_s() {
        assert_eq!(
            ::std::mem::size_of::<__pthread_mutex_s>(),
            40usize,
            concat!("Size of: ", stringify!(__pthread_mutex_s))
        );
        assert_eq!(
            ::std::mem::align_of::<__pthread_mutex_s>(),
            8usize,
            concat!("Alignment of ", stringify!(__pthread_mutex_s))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<__pthread_mutex_s>())).__lock as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_mutex_s),
                "::",
                stringify!(__lock)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<__pthread_mutex_s>())).__count as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_mutex_s),
                "::",
                stringify!(__count)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<__pthread_mutex_s>())).__owner as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_mutex_s),
                "::",
                stringify!(__owner)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<__pthread_mutex_s>())).__nusers as *const _ as usize },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_mutex_s),
                "::",
                stringify!(__nusers)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<__pthread_mutex_s>())).__kind as *const _ as usize },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_mutex_s),
                "::",
                stringify!(__kind)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<__pthread_mutex_s>())).__spins as *const _ as usize },
            20usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_mutex_s),
                "::",
                stringify!(__spins)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<__pthread_mutex_s>())).__elision as *const _ as usize },
            22usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_mutex_s),
                "::",
                stringify!(__elision)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<__pthread_mutex_s>())).__list as *const _ as usize },
            24usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_mutex_s),
                "::",
                stringify!(__list)
            )
        );
    }

    #[test]
    fn bindgen_test_layout___pthread_cond_s__bindgen_ty_1__bindgen_ty_1() {
        assert_eq!(
            ::std::mem::size_of::<__pthread_cond_s__bindgen_ty_1__bindgen_ty_1>(),
            8usize,
            concat!(
                "Size of: ",
                stringify!(__pthread_cond_s__bindgen_ty_1__bindgen_ty_1)
            )
        );
        assert_eq!(
            ::std::mem::align_of::<__pthread_cond_s__bindgen_ty_1__bindgen_ty_1>(),
            4usize,
            concat!(
                "Alignment of ",
                stringify!(__pthread_cond_s__bindgen_ty_1__bindgen_ty_1)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_cond_s__bindgen_ty_1__bindgen_ty_1>())).__low
                    as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_cond_s__bindgen_ty_1__bindgen_ty_1),
                "::",
                stringify!(__low)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_cond_s__bindgen_ty_1__bindgen_ty_1>())).__high
                    as *const _ as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_cond_s__bindgen_ty_1__bindgen_ty_1),
                "::",
                stringify!(__high)
            )
        );
    }

    #[test]
    fn bindgen_test_layout___pthread_cond_s__bindgen_ty_1() {
        assert_eq!(
            ::std::mem::size_of::<__pthread_cond_s__bindgen_ty_1>(),
            8usize,
            concat!("Size of: ", stringify!(__pthread_cond_s__bindgen_ty_1))
        );
        assert_eq!(
            ::std::mem::align_of::<__pthread_cond_s__bindgen_ty_1>(),
            8usize,
            concat!("Alignment of ", stringify!(__pthread_cond_s__bindgen_ty_1))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_cond_s__bindgen_ty_1>())).__wseq as *const _
                    as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_cond_s__bindgen_ty_1),
                "::",
                stringify!(__wseq)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_cond_s__bindgen_ty_1>())).__wseq32 as *const _
                    as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_cond_s__bindgen_ty_1),
                "::",
                stringify!(__wseq32)
            )
        );
    }

    #[test]
    fn bindgen_test_layout___pthread_cond_s__bindgen_ty_2__bindgen_ty_1() {
        assert_eq!(
            ::std::mem::size_of::<__pthread_cond_s__bindgen_ty_2__bindgen_ty_1>(),
            8usize,
            concat!(
                "Size of: ",
                stringify!(__pthread_cond_s__bindgen_ty_2__bindgen_ty_1)
            )
        );
        assert_eq!(
            ::std::mem::align_of::<__pthread_cond_s__bindgen_ty_2__bindgen_ty_1>(),
            4usize,
            concat!(
                "Alignment of ",
                stringify!(__pthread_cond_s__bindgen_ty_2__bindgen_ty_1)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_cond_s__bindgen_ty_2__bindgen_ty_1>())).__low
                    as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_cond_s__bindgen_ty_2__bindgen_ty_1),
                "::",
                stringify!(__low)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_cond_s__bindgen_ty_2__bindgen_ty_1>())).__high
                    as *const _ as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_cond_s__bindgen_ty_2__bindgen_ty_1),
                "::",
                stringify!(__high)
            )
        );
    }

    #[test]
    fn bindgen_test_layout___pthread_cond_s__bindgen_ty_2() {
        assert_eq!(
            ::std::mem::size_of::<__pthread_cond_s__bindgen_ty_2>(),
            8usize,
            concat!("Size of: ", stringify!(__pthread_cond_s__bindgen_ty_2))
        );
        assert_eq!(
            ::std::mem::align_of::<__pthread_cond_s__bindgen_ty_2>(),
            8usize,
            concat!("Alignment of ", stringify!(__pthread_cond_s__bindgen_ty_2))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_cond_s__bindgen_ty_2>())).__g1_start as *const _
                    as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_cond_s__bindgen_ty_2),
                "::",
                stringify!(__g1_start)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_cond_s__bindgen_ty_2>())).__g1_start32 as *const _
                    as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_cond_s__bindgen_ty_2),
                "::",
                stringify!(__g1_start32)
            )
        );
    }

    #[test]
    fn bindgen_test_layout___pthread_cond_s() {
        assert_eq!(
            ::std::mem::size_of::<__pthread_cond_s>(),
            48usize,
            concat!("Size of: ", stringify!(__pthread_cond_s))
        );
        assert_eq!(
            ::std::mem::align_of::<__pthread_cond_s>(),
            8usize,
            concat!("Alignment of ", stringify!(__pthread_cond_s))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<__pthread_cond_s>())).__g_refs as *const _ as usize },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_cond_s),
                "::",
                stringify!(__g_refs)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<__pthread_cond_s>())).__g_size as *const _ as usize },
            24usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_cond_s),
                "::",
                stringify!(__g_size)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_cond_s>())).__g1_orig_size as *const _ as usize
            },
            32usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_cond_s),
                "::",
                stringify!(__g1_orig_size)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<__pthread_cond_s>())).__wrefs as *const _ as usize },
            36usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_cond_s),
                "::",
                stringify!(__wrefs)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<__pthread_cond_s>())).__g_signals as *const _ as usize
            },
            40usize,
            concat!(
                "Offset of field: ",
                stringify!(__pthread_cond_s),
                "::",
                stringify!(__g_signals)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_pthread_mutexattr_t() {
        assert_eq!(
            ::std::mem::size_of::<pthread_mutexattr_t>(),
            4usize,
            concat!("Size of: ", stringify!(pthread_mutexattr_t))
        );
        assert_eq!(
            ::std::mem::align_of::<pthread_mutexattr_t>(),
            4usize,
            concat!("Alignment of ", stringify!(pthread_mutexattr_t))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_mutexattr_t>())).__size as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_mutexattr_t),
                "::",
                stringify!(__size)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_mutexattr_t>())).__align as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_mutexattr_t),
                "::",
                stringify!(__align)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_pthread_condattr_t() {
        assert_eq!(
            ::std::mem::size_of::<pthread_condattr_t>(),
            4usize,
            concat!("Size of: ", stringify!(pthread_condattr_t))
        );
        assert_eq!(
            ::std::mem::align_of::<pthread_condattr_t>(),
            4usize,
            concat!("Alignment of ", stringify!(pthread_condattr_t))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_condattr_t>())).__size as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_condattr_t),
                "::",
                stringify!(__size)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_condattr_t>())).__align as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_condattr_t),
                "::",
                stringify!(__align)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_pthread_attr_t() {
        assert_eq!(
            ::std::mem::size_of::<pthread_attr_t>(),
            56usize,
            concat!("Size of: ", stringify!(pthread_attr_t))
        );
        assert_eq!(
            ::std::mem::align_of::<pthread_attr_t>(),
            8usize,
            concat!("Alignment of ", stringify!(pthread_attr_t))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_attr_t>())).__size as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_attr_t),
                "::",
                stringify!(__size)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_attr_t>())).__align as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_attr_t),
                "::",
                stringify!(__align)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_pthread_mutex_t() {
        assert_eq!(
            ::std::mem::size_of::<pthread_mutex_t>(),
            40usize,
            concat!("Size of: ", stringify!(pthread_mutex_t))
        );
        assert_eq!(
            ::std::mem::align_of::<pthread_mutex_t>(),
            8usize,
            concat!("Alignment of ", stringify!(pthread_mutex_t))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_mutex_t>())).__data as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_mutex_t),
                "::",
                stringify!(__data)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_mutex_t>())).__size as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_mutex_t),
                "::",
                stringify!(__size)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_mutex_t>())).__align as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_mutex_t),
                "::",
                stringify!(__align)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_pthread_cond_t() {
        assert_eq!(
            ::std::mem::size_of::<pthread_cond_t>(),
            48usize,
            concat!("Size of: ", stringify!(pthread_cond_t))
        );
        assert_eq!(
            ::std::mem::align_of::<pthread_cond_t>(),
            8usize,
            concat!("Alignment of ", stringify!(pthread_cond_t))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_cond_t>())).__data as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_cond_t),
                "::",
                stringify!(__data)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_cond_t>())).__size as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_cond_t),
                "::",
                stringify!(__size)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_cond_t>())).__align as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_cond_t),
                "::",
                stringify!(__align)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_pthread_rwlock_t() {
        assert_eq!(
            ::std::mem::size_of::<pthread_rwlock_t>(),
            56usize,
            concat!("Size of: ", stringify!(pthread_rwlock_t))
        );
        assert_eq!(
            ::std::mem::align_of::<pthread_rwlock_t>(),
            8usize,
            concat!("Alignment of ", stringify!(pthread_rwlock_t))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_rwlock_t>())).__data as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_rwlock_t),
                "::",
                stringify!(__data)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_rwlock_t>())).__size as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_rwlock_t),
                "::",
                stringify!(__size)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_rwlock_t>())).__align as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_rwlock_t),
                "::",
                stringify!(__align)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_pthread_rwlockattr_t() {
        assert_eq!(
            ::std::mem::size_of::<pthread_rwlockattr_t>(),
            8usize,
            concat!("Size of: ", stringify!(pthread_rwlockattr_t))
        );
        assert_eq!(
            ::std::mem::align_of::<pthread_rwlockattr_t>(),
            8usize,
            concat!("Alignment of ", stringify!(pthread_rwlockattr_t))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_rwlockattr_t>())).__size as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_rwlockattr_t),
                "::",
                stringify!(__size)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<pthread_rwlockattr_t>())).__align as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_rwlockattr_t),
                "::",
                stringify!(__align)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_pthread_barrier_t() {
        assert_eq!(
            ::std::mem::size_of::<pthread_barrier_t>(),
            32usize,
            concat!("Size of: ", stringify!(pthread_barrier_t))
        );
        assert_eq!(
            ::std::mem::align_of::<pthread_barrier_t>(),
            8usize,
            concat!("Alignment of ", stringify!(pthread_barrier_t))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_barrier_t>())).__size as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_barrier_t),
                "::",
                stringify!(__size)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<pthread_barrier_t>())).__align as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_barrier_t),
                "::",
                stringify!(__align)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_pthread_barrierattr_t() {
        assert_eq!(
            ::std::mem::size_of::<pthread_barrierattr_t>(),
            4usize,
            concat!("Size of: ", stringify!(pthread_barrierattr_t))
        );
        assert_eq!(
            ::std::mem::align_of::<pthread_barrierattr_t>(),
            4usize,
            concat!("Alignment of ", stringify!(pthread_barrierattr_t))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<pthread_barrierattr_t>())).__size as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_barrierattr_t),
                "::",
                stringify!(__size)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<pthread_barrierattr_t>())).__align as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(pthread_barrierattr_t),
                "::",
                stringify!(__align)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_random_data() {
        assert_eq!(
            ::std::mem::size_of::<random_data>(),
            48usize,
            concat!("Size of: ", stringify!(random_data))
        );
        assert_eq!(
            ::std::mem::align_of::<random_data>(),
            8usize,
            concat!("Alignment of ", stringify!(random_data))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<random_data>())).fptr as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(random_data),
                "::",
                stringify!(fptr)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<random_data>())).rptr as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(random_data),
                "::",
                stringify!(rptr)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<random_data>())).state as *const _ as usize },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(random_data),
                "::",
                stringify!(state)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<random_data>())).rand_type as *const _ as usize },
            24usize,
            concat!(
                "Offset of field: ",
                stringify!(random_data),
                "::",
                stringify!(rand_type)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<random_data>())).rand_deg as *const _ as usize },
            28usize,
            concat!(
                "Offset of field: ",
                stringify!(random_data),
                "::",
                stringify!(rand_deg)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<random_data>())).rand_sep as *const _ as usize },
            32usize,
            concat!(
                "Offset of field: ",
                stringify!(random_data),
                "::",
                stringify!(rand_sep)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<random_data>())).end_ptr as *const _ as usize },
            40usize,
            concat!(
                "Offset of field: ",
                stringify!(random_data),
                "::",
                stringify!(end_ptr)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_drand48_data() {
        assert_eq!(
            ::std::mem::size_of::<drand48_data>(),
            24usize,
            concat!("Size of: ", stringify!(drand48_data))
        );
        assert_eq!(
            ::std::mem::align_of::<drand48_data>(),
            8usize,
            concat!("Alignment of ", stringify!(drand48_data))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<drand48_data>())).__x as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(drand48_data),
                "::",
                stringify!(__x)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<drand48_data>())).__old_x as *const _ as usize },
            6usize,
            concat!(
                "Offset of field: ",
                stringify!(drand48_data),
                "::",
                stringify!(__old_x)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<drand48_data>())).__c as *const _ as usize },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(drand48_data),
                "::",
                stringify!(__c)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<drand48_data>())).__init as *const _ as usize },
            14usize,
            concat!(
                "Offset of field: ",
                stringify!(drand48_data),
                "::",
                stringify!(__init)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<drand48_data>())).__a as *const _ as usize },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(drand48_data),
                "::",
                stringify!(__a)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_DATE_STRUCT() {
        assert_eq!(
            ::std::mem::size_of::<DATE_STRUCT>(),
            6usize,
            concat!("Size of: ", stringify!(DATE_STRUCT))
        );
        assert_eq!(
            ::std::mem::align_of::<DATE_STRUCT>(),
            2usize,
            concat!("Alignment of ", stringify!(DATE_STRUCT))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DATE_STRUCT>())).year as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(DATE_STRUCT),
                "::",
                stringify!(year)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DATE_STRUCT>())).month as *const _ as usize },
            2usize,
            concat!(
                "Offset of field: ",
                stringify!(DATE_STRUCT),
                "::",
                stringify!(month)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<DATE_STRUCT>())).day as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(DATE_STRUCT),
                "::",
                stringify!(day)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_TIME_STRUCT() {
        assert_eq!(
            ::std::mem::size_of::<TIME_STRUCT>(),
            6usize,
            concat!("Size of: ", stringify!(TIME_STRUCT))
        );
        assert_eq!(
            ::std::mem::align_of::<TIME_STRUCT>(),
            2usize,
            concat!("Alignment of ", stringify!(TIME_STRUCT))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIME_STRUCT>())).hour as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(TIME_STRUCT),
                "::",
                stringify!(hour)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIME_STRUCT>())).minute as *const _ as usize },
            2usize,
            concat!(
                "Offset of field: ",
                stringify!(TIME_STRUCT),
                "::",
                stringify!(minute)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIME_STRUCT>())).second as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(TIME_STRUCT),
                "::",
                stringify!(second)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_TIMESTAMP_STRUCT() {
        assert_eq!(
            ::std::mem::size_of::<TIMESTAMP_STRUCT>(),
            16usize,
            concat!("Size of: ", stringify!(TIMESTAMP_STRUCT))
        );
        assert_eq!(
            ::std::mem::align_of::<TIMESTAMP_STRUCT>(),
            4usize,
            concat!("Alignment of ", stringify!(TIMESTAMP_STRUCT))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIMESTAMP_STRUCT>())).year as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT),
                "::",
                stringify!(year)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIMESTAMP_STRUCT>())).month as *const _ as usize },
            2usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT),
                "::",
                stringify!(month)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIMESTAMP_STRUCT>())).day as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT),
                "::",
                stringify!(day)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIMESTAMP_STRUCT>())).hour as *const _ as usize },
            6usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT),
                "::",
                stringify!(hour)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIMESTAMP_STRUCT>())).minute as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT),
                "::",
                stringify!(minute)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIMESTAMP_STRUCT>())).second as *const _ as usize },
            10usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT),
                "::",
                stringify!(second)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIMESTAMP_STRUCT>())).fraction as *const _ as usize },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT),
                "::",
                stringify!(fraction)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_TIMESTAMP_STRUCT_EXT() {
        assert_eq!(
            ::std::mem::size_of::<TIMESTAMP_STRUCT_EXT>(),
            20usize,
            concat!("Size of: ", stringify!(TIMESTAMP_STRUCT_EXT))
        );
        assert_eq!(
            ::std::mem::align_of::<TIMESTAMP_STRUCT_EXT>(),
            4usize,
            concat!("Alignment of ", stringify!(TIMESTAMP_STRUCT_EXT))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT>())).year as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT),
                "::",
                stringify!(year)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT>())).month as *const _ as usize },
            2usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT),
                "::",
                stringify!(month)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT>())).day as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT),
                "::",
                stringify!(day)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT>())).hour as *const _ as usize },
            6usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT),
                "::",
                stringify!(hour)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT>())).minute as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT),
                "::",
                stringify!(minute)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT>())).second as *const _ as usize },
            10usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT),
                "::",
                stringify!(second)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT>())).fraction as *const _ as usize
            },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT),
                "::",
                stringify!(fraction)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT>())).fraction2 as *const _ as usize
            },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT),
                "::",
                stringify!(fraction2)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_TIMESTAMP_STRUCT_EXT_TZ() {
        assert_eq!(
            ::std::mem::size_of::<TIMESTAMP_STRUCT_EXT_TZ>(),
            24usize,
            concat!("Size of: ", stringify!(TIMESTAMP_STRUCT_EXT_TZ))
        );
        assert_eq!(
            ::std::mem::align_of::<TIMESTAMP_STRUCT_EXT_TZ>(),
            4usize,
            concat!("Alignment of ", stringify!(TIMESTAMP_STRUCT_EXT_TZ))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT_TZ>())).year as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT_TZ),
                "::",
                stringify!(year)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT_TZ>())).month as *const _ as usize
            },
            2usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT_TZ),
                "::",
                stringify!(month)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT_TZ>())).day as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT_TZ),
                "::",
                stringify!(day)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT_TZ>())).hour as *const _ as usize
            },
            6usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT_TZ),
                "::",
                stringify!(hour)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT_TZ>())).minute as *const _ as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT_TZ),
                "::",
                stringify!(minute)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT_TZ>())).second as *const _ as usize
            },
            10usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT_TZ),
                "::",
                stringify!(second)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT_TZ>())).fraction as *const _ as usize
            },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT_TZ),
                "::",
                stringify!(fraction)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT_TZ>())).fraction2 as *const _ as usize
            },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT_TZ),
                "::",
                stringify!(fraction2)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT_TZ>())).timezone_hour as *const _
                    as usize
            },
            20usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT_TZ),
                "::",
                stringify!(timezone_hour)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<TIMESTAMP_STRUCT_EXT_TZ>())).timezone_minute as *const _
                    as usize
            },
            22usize,
            concat!(
                "Offset of field: ",
                stringify!(TIMESTAMP_STRUCT_EXT_TZ),
                "::",
                stringify!(timezone_minute)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_tagSQL_YEAR_MONTH() {
        assert_eq!(
            ::std::mem::size_of::<tagSQL_YEAR_MONTH>(),
            8usize,
            concat!("Size of: ", stringify!(tagSQL_YEAR_MONTH))
        );
        assert_eq!(
            ::std::mem::align_of::<tagSQL_YEAR_MONTH>(),
            4usize,
            concat!("Alignment of ", stringify!(tagSQL_YEAR_MONTH))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<tagSQL_YEAR_MONTH>())).year as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_YEAR_MONTH),
                "::",
                stringify!(year)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<tagSQL_YEAR_MONTH>())).month as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_YEAR_MONTH),
                "::",
                stringify!(month)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_tagSQL_DAY_SECOND() {
        assert_eq!(
            ::std::mem::size_of::<tagSQL_DAY_SECOND>(),
            20usize,
            concat!("Size of: ", stringify!(tagSQL_DAY_SECOND))
        );
        assert_eq!(
            ::std::mem::align_of::<tagSQL_DAY_SECOND>(),
            4usize,
            concat!("Alignment of ", stringify!(tagSQL_DAY_SECOND))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<tagSQL_DAY_SECOND>())).day as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_DAY_SECOND),
                "::",
                stringify!(day)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<tagSQL_DAY_SECOND>())).hour as *const _ as usize },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_DAY_SECOND),
                "::",
                stringify!(hour)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<tagSQL_DAY_SECOND>())).minute as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_DAY_SECOND),
                "::",
                stringify!(minute)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<tagSQL_DAY_SECOND>())).second as *const _ as usize },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_DAY_SECOND),
                "::",
                stringify!(second)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<tagSQL_DAY_SECOND>())).fraction as *const _ as usize },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_DAY_SECOND),
                "::",
                stringify!(fraction)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_tagSQL_INTERVAL_STRUCT__bindgen_ty_1() {
        assert_eq!(
            ::std::mem::size_of::<tagSQL_INTERVAL_STRUCT__bindgen_ty_1>(),
            20usize,
            concat!(
                "Size of: ",
                stringify!(tagSQL_INTERVAL_STRUCT__bindgen_ty_1)
            )
        );
        assert_eq!(
            ::std::mem::align_of::<tagSQL_INTERVAL_STRUCT__bindgen_ty_1>(),
            4usize,
            concat!(
                "Alignment of ",
                stringify!(tagSQL_INTERVAL_STRUCT__bindgen_ty_1)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<tagSQL_INTERVAL_STRUCT__bindgen_ty_1>())).year_month
                    as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_INTERVAL_STRUCT__bindgen_ty_1),
                "::",
                stringify!(year_month)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<tagSQL_INTERVAL_STRUCT__bindgen_ty_1>())).day_second
                    as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_INTERVAL_STRUCT__bindgen_ty_1),
                "::",
                stringify!(day_second)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_tagSQL_INTERVAL_STRUCT() {
        assert_eq!(
            ::std::mem::size_of::<tagSQL_INTERVAL_STRUCT>(),
            28usize,
            concat!("Size of: ", stringify!(tagSQL_INTERVAL_STRUCT))
        );
        assert_eq!(
            ::std::mem::align_of::<tagSQL_INTERVAL_STRUCT>(),
            4usize,
            concat!("Alignment of ", stringify!(tagSQL_INTERVAL_STRUCT))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<tagSQL_INTERVAL_STRUCT>())).interval_type as *const _
                    as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_INTERVAL_STRUCT),
                "::",
                stringify!(interval_type)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<tagSQL_INTERVAL_STRUCT>())).interval_sign as *const _
                    as usize
            },
            4usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_INTERVAL_STRUCT),
                "::",
                stringify!(interval_sign)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<tagSQL_INTERVAL_STRUCT>())).intval as *const _ as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_INTERVAL_STRUCT),
                "::",
                stringify!(intval)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_tagSQL_NUMERIC_STRUCT() {
        assert_eq!(
            ::std::mem::size_of::<tagSQL_NUMERIC_STRUCT>(),
            19usize,
            concat!("Size of: ", stringify!(tagSQL_NUMERIC_STRUCT))
        );
        assert_eq!(
            ::std::mem::align_of::<tagSQL_NUMERIC_STRUCT>(),
            1usize,
            concat!("Alignment of ", stringify!(tagSQL_NUMERIC_STRUCT))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<tagSQL_NUMERIC_STRUCT>())).precision as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_NUMERIC_STRUCT),
                "::",
                stringify!(precision)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<tagSQL_NUMERIC_STRUCT>())).scale as *const _ as usize },
            1usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_NUMERIC_STRUCT),
                "::",
                stringify!(scale)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<tagSQL_NUMERIC_STRUCT>())).sign as *const _ as usize },
            2usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_NUMERIC_STRUCT),
                "::",
                stringify!(sign)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<tagSQL_NUMERIC_STRUCT>())).val as *const _ as usize },
            3usize,
            concat!(
                "Offset of field: ",
                stringify!(tagSQL_NUMERIC_STRUCT),
                "::",
                stringify!(val)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_SQLDECIMAL64__bindgen_ty_1() {
        assert_eq!(
            ::std::mem::size_of::<SQLDECIMAL64__bindgen_ty_1>(),
            8usize,
            concat!("Size of: ", stringify!(SQLDECIMAL64__bindgen_ty_1))
        );
        assert_eq!(
            ::std::mem::align_of::<SQLDECIMAL64__bindgen_ty_1>(),
            8usize,
            concat!("Alignment of ", stringify!(SQLDECIMAL64__bindgen_ty_1))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<SQLDECIMAL64__bindgen_ty_1>())).dummy as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(SQLDECIMAL64__bindgen_ty_1),
                "::",
                stringify!(dummy)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<SQLDECIMAL64__bindgen_ty_1>())).dec64 as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(SQLDECIMAL64__bindgen_ty_1),
                "::",
                stringify!(dec64)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_SQLDECIMAL64() {
        assert_eq!(
            ::std::mem::size_of::<SQLDECIMAL64>(),
            8usize,
            concat!("Size of: ", stringify!(SQLDECIMAL64))
        );
        assert_eq!(
            ::std::mem::align_of::<SQLDECIMAL64>(),
            8usize,
            concat!("Alignment of ", stringify!(SQLDECIMAL64))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SQLDECIMAL64>())).udec64 as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(SQLDECIMAL64),
                "::",
                stringify!(udec64)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_SQLDECIMAL128__bindgen_ty_1() {
        assert_eq!(
            ::std::mem::size_of::<SQLDECIMAL128__bindgen_ty_1>(),
            16usize,
            concat!("Size of: ", stringify!(SQLDECIMAL128__bindgen_ty_1))
        );
        assert_eq!(
            ::std::mem::align_of::<SQLDECIMAL128__bindgen_ty_1>(),
            8usize,
            concat!("Alignment of ", stringify!(SQLDECIMAL128__bindgen_ty_1))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<SQLDECIMAL128__bindgen_ty_1>())).dummy as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(SQLDECIMAL128__bindgen_ty_1),
                "::",
                stringify!(dummy)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<SQLDECIMAL128__bindgen_ty_1>())).dec128 as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(SQLDECIMAL128__bindgen_ty_1),
                "::",
                stringify!(dec128)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_SQLDECIMAL128() {
        assert_eq!(
            ::std::mem::size_of::<SQLDECIMAL128>(),
            16usize,
            concat!("Size of: ", stringify!(SQLDECIMAL128))
        );
        assert_eq!(
            ::std::mem::align_of::<SQLDECIMAL128>(),
            8usize,
            concat!("Alignment of ", stringify!(SQLDECIMAL128))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<SQLDECIMAL128>())).udec128 as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(SQLDECIMAL128),
                "::",
                stringify!(udec128)
            )
        );
    }

    #[test]
    fn bindgen_test_layout__TAGGUID() {
        assert_eq!(
            ::std::mem::size_of::<_TAGGUID>(),
            24usize,
            concat!("Size of: ", stringify!(_TAGGUID))
        );
        assert_eq!(
            ::std::mem::align_of::<_TAGGUID>(),
            8usize,
            concat!("Alignment of ", stringify!(_TAGGUID))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<_TAGGUID>())).Data1 as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(_TAGGUID),
                "::",
                stringify!(Data1)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<_TAGGUID>())).Data2 as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(_TAGGUID),
                "::",
                stringify!(Data2)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<_TAGGUID>())).Data3 as *const _ as usize },
            10usize,
            concat!(
                "Offset of field: ",
                stringify!(_TAGGUID),
                "::",
                stringify!(Data3)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<_TAGGUID>())).Data4 as *const _ as usize },
            12usize,
            concat!(
                "Offset of field: ",
                stringify!(_TAGGUID),
                "::",
                stringify!(Data4)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_tagODBC_VS_ARGS__bindgen_ty_1() {
        assert_eq!(
            ::std::mem::size_of::<tagODBC_VS_ARGS__bindgen_ty_1>(),
            8usize,
            concat!("Size of: ", stringify!(tagODBC_VS_ARGS__bindgen_ty_1))
        );
        assert_eq!(
            ::std::mem::align_of::<tagODBC_VS_ARGS__bindgen_ty_1>(),
            8usize,
            concat!("Alignment of ", stringify!(tagODBC_VS_ARGS__bindgen_ty_1))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<tagODBC_VS_ARGS__bindgen_ty_1>())).wszArg as *const _
                    as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(tagODBC_VS_ARGS__bindgen_ty_1),
                "::",
                stringify!(wszArg)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<tagODBC_VS_ARGS__bindgen_ty_1>())).szArg as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(tagODBC_VS_ARGS__bindgen_ty_1),
                "::",
                stringify!(szArg)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_tagODBC_VS_ARGS__bindgen_ty_2() {
        assert_eq!(
            ::std::mem::size_of::<tagODBC_VS_ARGS__bindgen_ty_2>(),
            8usize,
            concat!("Size of: ", stringify!(tagODBC_VS_ARGS__bindgen_ty_2))
        );
        assert_eq!(
            ::std::mem::align_of::<tagODBC_VS_ARGS__bindgen_ty_2>(),
            8usize,
            concat!("Alignment of ", stringify!(tagODBC_VS_ARGS__bindgen_ty_2))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<tagODBC_VS_ARGS__bindgen_ty_2>())).wszCorrelation as *const _
                    as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(tagODBC_VS_ARGS__bindgen_ty_2),
                "::",
                stringify!(wszCorrelation)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<tagODBC_VS_ARGS__bindgen_ty_2>())).szCorrelation as *const _
                    as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(tagODBC_VS_ARGS__bindgen_ty_2),
                "::",
                stringify!(szCorrelation)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_tagODBC_VS_ARGS() {
        assert_eq!(
            ::std::mem::size_of::<tagODBC_VS_ARGS>(),
            40usize,
            concat!("Size of: ", stringify!(tagODBC_VS_ARGS))
        );
        assert_eq!(
            ::std::mem::align_of::<tagODBC_VS_ARGS>(),
            8usize,
            concat!("Alignment of ", stringify!(tagODBC_VS_ARGS))
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<tagODBC_VS_ARGS>())).pguidEvent as *const _ as usize },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(tagODBC_VS_ARGS),
                "::",
                stringify!(pguidEvent)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<tagODBC_VS_ARGS>())).dwFlags as *const _ as usize },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(tagODBC_VS_ARGS),
                "::",
                stringify!(dwFlags)
            )
        );
        assert_eq!(
            unsafe { &(*(::std::ptr::null::<tagODBC_VS_ARGS>())).RetCode as *const _ as usize },
            32usize,
            concat!(
                "Offset of field: ",
                stringify!(tagODBC_VS_ARGS),
                "::",
                stringify!(RetCode)
            )
        );
    }

    #[test]
    fn bindgen_test_layout_SQL_NET_STATS() {
        assert_eq!(
            ::std::mem::size_of::<SQL_NET_STATS>(),
            48usize,
            concat!("Size of: ", stringify!(SQL_NET_STATS))
        );
        assert_eq!(
            ::std::mem::align_of::<SQL_NET_STATS>(),
            8usize,
            concat!("Alignment of ", stringify!(SQL_NET_STATS))
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<SQL_NET_STATS>())).iNetStatsLength as *const _ as usize
            },
            0usize,
            concat!(
                "Offset of field: ",
                stringify!(SQL_NET_STATS),
                "::",
                stringify!(iNetStatsLength)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<SQL_NET_STATS>())).uiNetStatsServerTime as *const _ as usize
            },
            8usize,
            concat!(
                "Offset of field: ",
                stringify!(SQL_NET_STATS),
                "::",
                stringify!(uiNetStatsServerTime)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<SQL_NET_STATS>())).uiNetStatsNetworkTime as *const _ as usize
            },
            16usize,
            concat!(
                "Offset of field: ",
                stringify!(SQL_NET_STATS),
                "::",
                stringify!(uiNetStatsNetworkTime)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<SQL_NET_STATS>())).uiNetStatsBytesSent as *const _ as usize
            },
            24usize,
            concat!(
                "Offset of field: ",
                stringify!(SQL_NET_STATS),
                "::",
                stringify!(uiNetStatsBytesSent)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<SQL_NET_STATS>())).uiNetStatsBytesReceived as *const _
                    as usize
            },
            32usize,
            concat!(
                "Offset of field: ",
                stringify!(SQL_NET_STATS),
                "::",
                stringify!(uiNetStatsBytesReceived)
            )
        );
        assert_eq!(
            unsafe {
                &(*(::std::ptr::null::<SQL_NET_STATS>())).uiNetStatsRoundTrips as *const _ as usize
            },
            40usize,
            concat!(
                "Offset of field: ",
                stringify!(SQL_NET_STATS),
                "::",
                stringify!(uiNetStatsRoundTrips)
            )
        );
    }
}
