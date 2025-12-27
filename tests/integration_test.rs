use std::path::PathBuf;

#[test]
fn test_dependencies_fixture() {
    // This test uses the dependencies fixture from tests/fixtures/dependencies
    // It should correctly identify:
    // - Unused dependencies: fs-extra, mocha, stream
    // - Used dependencies: @sindresorhus/is, has, JSONStream, @tootallnate/once, ansi-regex
    // - Unused files: unused-module.ts (should be reported as it's not imported)

    let fixture_dir = PathBuf::from("tests/fixtures/dependencies");
    assert!(fixture_dir.exists());

    // Entry point
    let entry = fixture_dir.join("entry.ts");
    assert!(entry.exists());

    // Files that should be reachable
    let my_module = fixture_dir.join("my-module.ts");
    assert!(my_module.exists());

    let unused_module = fixture_dir.join("unused-module.ts");
    assert!(unused_module.exists());

    // TODO: Run actual sweepr analysis and verify results
    // For now, just verify the fixture structure
}
