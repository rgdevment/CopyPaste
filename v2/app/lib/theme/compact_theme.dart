import 'package:flutter/material.dart';

import 'app_theme_data.dart';
import 'dark_theme.dart';
import 'light_theme.dart';

class CompactTheme extends AppThemeData {
  @override
  String get id => 'compact';

  @override
  String get name => 'Compact';

  @override
  AppThemeColorScheme get light => lightColorScheme;

  @override
  AppThemeColorScheme get dark => darkColorScheme;

  @override
  AppThemeTypography get typography => const AppThemeTypography(
    fontFamily: 'Inter',
    cardContent: TextStyle(
      fontSize: 13,
      height: 1.5,
      fontWeight: FontWeight.w400,
    ),
    cardHeader: TextStyle(fontSize: 10.5, fontWeight: FontWeight.w700),
    cardLabel: TextStyle(
      fontSize: 10.5,
      fontWeight: FontWeight.w700,
      letterSpacing: 0.06,
    ),
    cardFooter: TextStyle(fontSize: 10),
    cardTimestamp: TextStyle(fontSize: 10),
    searchInput: TextStyle(fontSize: 13, fontWeight: FontWeight.w400),
    filterChip: TextStyle(fontSize: 11, fontWeight: FontWeight.w500),
    filterTabChip: TextStyle(fontSize: 11, fontWeight: FontWeight.w500),
    toolbarButton: TextStyle(fontSize: 11),
    tabLabel: TextStyle(fontSize: 12, fontWeight: FontWeight.w500),
    emptyState: TextStyle(fontSize: 12.5),
    emptyStateIcon: TextStyle(fontSize: 32),
    branding: TextStyle(fontSize: 11, fontWeight: FontWeight.w500),
  );

  @override
  AppThemeSpacing get spacing => const AppThemeSpacing(
    xs: 2,
    sm: 4,
    md: 8,
    lg: 12,
    xl: 16,
    cardPadding: EdgeInsets.symmetric(horizontal: 13, vertical: 11),
    cardGap: 5,
    listPadding: EdgeInsets.symmetric(horizontal: 8),
    searchBarPadding: EdgeInsets.fromLTRB(12, 12, 12, 10),
    titleBarHeight: 56,
    bottomBarHeight: 34,
    filterTabBarPadding: EdgeInsets.fromLTRB(12, 0, 12, 10),
    filterTabBarHeight: 34,
  );

  @override
  AppThemeRadii get radii => const AppThemeRadii(
    xs: 4,
    sm: 6,
    md: 8,
    lg: 12,
    card: 9,
    chip: 999,
    searchBox: 999,
    button: 6,
    thumbnail: 6,
  );

  @override
  AppThemeSizing get sizing => const AppThemeSizing(
    cardMinHeight: 52,
    cardImageHeight: 110,
    cardMinLines: 2,
    cardMaxLines: 5,
    chipHeight: 28,
    iconSizeXs: 9,
    iconSizeSm: 10,
    iconSizeMd: 14,
    iconSizeLg: 18,
    titleBarIconSize: 16,
    toolbarIconSize: 14,
    colorIndicatorWidth: 3,
    colorDotSize: 12,
    searchBoxHeight: 34,
    cardTypeIconContainerSize: 34,
  );

  @override
  AppThemeIcons get icons => const AppThemeIcons(
    text: Icons.text_snippet_outlined,
    image: Icons.image_outlined,
    link: Icons.link_rounded,
    file: Icons.insert_drive_file_outlined,
    folder: Icons.folder_outlined,
    audio: Icons.music_note_rounded,
    video: Icons.videocam_outlined,
    unknown: Icons.help_outline_rounded,
    pin: Icons.push_pin_outlined,
    pinFilled: Icons.push_pin_rounded,
    delete: Icons.delete_outline_rounded,
    edit: Icons.edit_outlined,
    copy: Icons.content_copy_rounded,
    paste: Icons.content_paste_rounded,
    search: Icons.search_rounded,
    filter: Icons.tune_rounded,
    close: Icons.close_rounded,
    settings: Icons.settings_outlined,
    help: Icons.help_outline_rounded,
    recent: Icons.access_time_rounded,
    clear: Icons.clear_all_rounded,
    warning: Icons.warning_amber_rounded,
    colorLabel: Icons.circle,
  );

  @override
  AppThemeCardStyle get cardStyle => const AppThemeCardStyle(
    elevation: 0,
    hoverElevation: 0,
    borderWidth: 1.0,
    colorIndicatorBorderRadius: BorderRadius.only(
      topLeft: Radius.circular(9),
      bottomLeft: Radius.circular(9),
    ),
    contentLineHeight: 1.45,
    headerOpacity: 0.7,
    footerOpacity: 0.4,
    timestampOpacity: 0.38,
    contentOpacity: 0.82,
    hoverActionOpacity: 0.55,
    appSourceOpacity: 0.45,
  );

  @override
  AppThemeFilterStyle get filterStyle => const AppThemeFilterStyle(
    chipSpacing: 6,
    chipPadding: EdgeInsets.symmetric(horizontal: 8, vertical: 4),
    selectedOpacity: 1.0,
    unselectedOpacity: 0.5,
    animationDuration: Duration(milliseconds: 200),
  );

  @override
  AppThemeSearchStyle get searchStyle => const AppThemeSearchStyle(
    debounceDuration: Duration(milliseconds: 300),
    padding: EdgeInsets.symmetric(horizontal: 14, vertical: 8),
    iconOpacity: 0.4,
  );

  @override
  AppThemeToolbarStyle get toolbarStyle => const AppThemeToolbarStyle(
    buttonSpacing: 2,
    buttonPadding: EdgeInsets.all(4),
    iconOpacity: 0.6,
    hoverOpacity: 0.9,
  );
}
