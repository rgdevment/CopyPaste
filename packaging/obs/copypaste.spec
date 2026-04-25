Name:           copypaste
Version:        @VERSION@
Release:        0
Summary:        Free, open source clipboard manager and clipboard history tool
License:        GPL-3.0-only
Group:          Productivity/Utilities
URL:            https://github.com/rgdevment/CopyPaste
Source0:        CopyPaste-%{version}-linux-x64.tar.gz
BuildRequires:  desktop-file-utils
BuildRequires:  hicolor-icon-theme
ExclusiveArch:  x86_64

%global __brp_check_rpaths %{nil}
%global __requires_exclude_from ^/opt/copypaste/.*$
%global __provides_exclude_from ^/opt/copypaste/.*$
Requires:       hicolor-icon-theme
%if 0%{?suse_version}
Requires:       libayatana-appindicator3-1
Requires:       libkeybinder-3_0-0
Requires:       libgtk-3-0
Requires:       libX11-6
Requires:       libXtst6
%else
Requires:       libayatana-appindicator-gtk3
Requires:       keybinder3
Requires:       gtk3
Requires:       libX11
Requires:       libXtst
%endif

%description
CopyPaste is a free, open source, local-first clipboard manager and
clipboard history tool for X11 sessions on Linux. No telemetry, no
accounts, no cloud — your clipboard data never leaves your computer.

%global debug_package %{nil}

%prep
%setup -q -n CopyPaste-%{version}-linux-x64

%build

%install
install -d %{buildroot}/opt/copypaste
cp -a bundle/. %{buildroot}/opt/copypaste/
chmod 0755 %{buildroot}/opt/copypaste/copypaste
find %{buildroot}/opt/copypaste/lib -type f -name '*.so' -exec chmod 0755 {} +
install -d %{buildroot}%{_bindir}
ln -s /opt/copypaste/copypaste %{buildroot}%{_bindir}/copypaste
install -Dm644 packaging/com.rgdevment.copypaste.desktop \
    %{buildroot}%{_datadir}/applications/com.rgdevment.copypaste.desktop
install -Dm644 packaging/icon_app_256.png \
    %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/com.rgdevment.copypaste.png
desktop-file-validate %{buildroot}%{_datadir}/applications/com.rgdevment.copypaste.desktop

%files
%license LICENSE
/opt/copypaste
%{_bindir}/copypaste
%{_datadir}/applications/com.rgdevment.copypaste.desktop
%{_datadir}/icons/hicolor/256x256/apps/com.rgdevment.copypaste.png

%changelog
* Thu Apr 23 2026 rgdevment <rgdevment@users.noreply.github.com> - @VERSION@-0
- Automated release from GitHub Actions
