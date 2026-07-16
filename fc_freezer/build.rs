fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();

        let manifest_xml = concat!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n",
        "<assembly xmlns=\"urn:schemas-microsoft-com:asm.v1\" manifestVersion=\"1.0\">\n",
        "  <trustInfo xmlns=\"urn:schemas-microsoft-com:asm.v3\">\n",
        "    <security>\n",
        "      <requestedPrivileges>\n",
        "        <requestedExecutionLevel level=\"requireAdministrator\" uiAccess=\"false\"/>\n",
        "      </requestedPrivileges>\n",
        "    </security>\n",
        "  </trustInfo>\n",
        "</assembly>"
        );

        res.set_manifest(manifest_xml);
        res.compile().unwrap();
    }
}
