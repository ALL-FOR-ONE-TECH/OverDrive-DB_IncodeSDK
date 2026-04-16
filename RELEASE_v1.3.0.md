# OverDrive-DB SDK v1.3.0 Release Notes

**Release Date**: April 13, 2026

## 🎯 What's New

This release brings the SDK in sync with OverDrive-DB v1.3.0, featuring the new TUI v3 redesign and enhanced REST API capabilities.

## 📦 Package Updates

### Node.js (@all-for-one-tech/overdrive-db)
- **Version**: 1.0.0 → 1.3.0
- **npm**: `npm install @all-for-one-tech/overdrive-db@1.3.0`

### Python (overdrive-db)
- **Version**: 1.3.0 (already current)
- **PyPI**: `pip install overdrive-db==1.3.0`

### Java (com.afot:overdrive-db)
- **Version**: 1.3.0 (already current)
- **Maven Central**: 
  ```xml
  <dependency>
      <groupId>com.afot</groupId>
      <artifactId>overdrive-db</artifactId>
      <version>1.3.0</version>
  </dependency>
  ```

### Go (github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/go)
- **Version**: Tagged as `go/v1.3.0`
- **Install**: `go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/go@v1.3.0`

## 🚀 Publishing Instructions

### Prerequisites

1. **Node.js**: Ensure you're logged into npm
   ```bash
   npm login
   ```

2. **Python**: Ensure you have `twine` installed
   ```bash
   pip install twine build
   ```

3. **Java**: Ensure Maven settings.xml is configured with credentials

4. **Go**: Ensure you have push access to the repository

### Automated Publishing

Run the automated publish script from the `IncodeSDK` directory:

```powershell
cd IncodeSDK
powershell -File publish-all.ps1
```

This will:
1. Publish Node.js SDK to npm
2. Publish Python SDK to PyPI
3. Publish Java SDK to Maven Central
4. Display instructions for Go tag

### Manual Publishing

#### Node.js

```bash
cd IncodeSDK/nodejs
npm publish --access public
```

#### Python

```bash
cd IncodeSDK/python
python -m build
twine upload dist/* --skip-existing
```

#### Java

```bash
cd IncodeSDK/java
mvn clean deploy -s settings.xml -DskipTests
```

#### Go

```bash
# From the SDK repository root
git tag go/v1.3.0
git push origin go/v1.3.0
```

## 📝 Changelog

### Added
- Updated descriptions to reflect TUI v3 redesign
- Updated descriptions to reflect full REST API SQL support
- Synchronized all package versions to 1.3.0

### Changed
- Node.js package description updated
- Python package description updated
- All packages now reference the latest OverDrive-DB features

## 🔗 Links

- **Main Repository**: https://github.com/afot/overdrivedb
- **SDK Repository**: https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK
- **Documentation**: https://overdrive-db.com/docs
- **npm Package**: https://www.npmjs.com/package/@all-for-one-tech/overdrive-db
- **PyPI Package**: https://pypi.org/project/overdrive-db/
- **Maven Central**: https://central.sonatype.com/artifact/com.afot/overdrive-db

## 📞 Support

- **Issues**: https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/issues
- **Email**: admin@afot.in
- **Website**: https://overdrive-db.com

---

**Full Changelog**: https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/compare/v1.2.1...v1.3.0
