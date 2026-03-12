import 'package:flutter/material.dart';

abstract class AppThemeData {
  String get id;
  String get name;

  AppThemeColorScheme get light;
  AppThemeColorScheme get dark;

  AppThemeTypography get typography;
  AppThemeSpacing get spacing;
  AppThemeRadii get radii;
  AppThemeSizing get sizing;
  AppThemeIcons get icons;
  AppThemeCardStyle get cardStyle;
  AppThemeFilterStyle get filterStyle;
  AppThemeSearchStyle get searchStyle;
  AppThemeToolbarStyle get toolbarStyle;
}

class AppThemeColorScheme {
  const AppThemeColorScheme({
    required this.surface,
    required this.surfaceVariant,
    required this.background,
    required this.onSurface,
    required this.onSurfaceVariant,
    required this.onSurfaceMuted,
    required this.onSurfaceSubtle,
    required this.primary,
    required this.onPrimary,
    required this.cardBackground,
    required this.cardBorder,
    required this.searchBackground,
    required this.searchBorder,
    required this.divider,
    required this.danger,
    required this.warning,
    required this.accentRed,
    required this.accentGreen,
    required this.accentPurple,
    required this.accentYellow,
    required this.accentBlue,
    required this.accentOrange,
  });

  final Color surface;
  final Color surfaceVariant;
  final Color background;
  final Color onSurface;
  final Color onSurfaceVariant;
  final Color onSurfaceMuted;
  final Color onSurfaceSubtle;
  final Color primary;
  final Color onPrimary;
  final Color cardBackground;
  final Color cardBorder;
  final Color searchBackground;
  final Color searchBorder;
  final Color divider;
  final Color danger;
  final Color warning;
  final Color accentRed;
  final Color accentGreen;
  final Color accentPurple;
  final Color accentYellow;
  final Color accentBlue;
  final Color accentOrange;

  Color accentForIndex(int index) => switch (index) {
    1 => accentRed,
    2 => accentGreen,
    3 => accentPurple,
    4 => accentYellow,
    5 => accentBlue,
    6 => accentOrange,
    _ => Colors.transparent,
  };
}

class AppThemeTypography {
  const AppThemeTypography({
    required this.fontFamily,
    required this.cardContent,
    required this.cardHeader,
    required this.cardLabel,
    required this.cardFooter,
    required this.cardTimestamp,
    required this.searchInput,
    required this.filterChip,
    required this.filterTabChip,
    required this.toolbarButton,
    required this.tabLabel,
    required this.emptyState,
    required this.emptyStateIcon,
    required this.branding,
  });

  final String fontFamily;
  final TextStyle cardContent;
  final TextStyle cardHeader;
  final TextStyle cardLabel;
  final TextStyle cardFooter;
  final TextStyle cardTimestamp;
  final TextStyle searchInput;
  final TextStyle filterChip;
  final TextStyle filterTabChip;
  final TextStyle toolbarButton;
  final TextStyle tabLabel;
  final TextStyle emptyState;
  final TextStyle emptyStateIcon;
  final TextStyle branding;
}

class AppThemeSpacing {
  const AppThemeSpacing({
    required this.xs,
    required this.sm,
    required this.md,
    required this.lg,
    required this.xl,
    required this.cardPadding,
    required this.cardGap,
    required this.listPadding,
    required this.searchBarPadding,
    required this.titleBarHeight,
    required this.bottomBarHeight,
    required this.filterTabBarPadding,
    required this.filterTabBarHeight,
  });

  final double xs;
  final double sm;
  final double md;
  final double lg;
  final double xl;
  final EdgeInsets cardPadding;
  final double cardGap;
  final EdgeInsets listPadding;
  final EdgeInsets searchBarPadding;
  final double titleBarHeight;
  final double bottomBarHeight;
  final EdgeInsets filterTabBarPadding;
  final double filterTabBarHeight;
}

class AppThemeRadii {
  const AppThemeRadii({
    required this.xs,
    required this.sm,
    required this.md,
    required this.lg,
    required this.card,
    required this.chip,
    required this.searchBox,
    required this.button,
    required this.thumbnail,
  });

  final double xs;
  final double sm;
  final double md;
  final double lg;
  final double card;
  final double chip;
  final double searchBox;
  final double button;
  final double thumbnail;
}

class AppThemeSizing {
  const AppThemeSizing({
    required this.cardMinHeight,
    required this.cardImageHeight,
    required this.cardMinLines,
    required this.cardMaxLines,
    required this.chipHeight,
    required this.iconSizeXs,
    required this.iconSizeSm,
    required this.iconSizeMd,
    required this.iconSizeLg,
    required this.titleBarIconSize,
    required this.toolbarIconSize,
    required this.colorIndicatorWidth,
    required this.colorDotSize,
    required this.searchBoxHeight,
    required this.cardTypeIconContainerSize,
  });

  final double cardMinHeight;
  final double cardImageHeight;
  final int cardMinLines;
  final int cardMaxLines;
  final double chipHeight;
  final double iconSizeXs;
  final double iconSizeSm;
  final double iconSizeMd;
  final double iconSizeLg;
  final double titleBarIconSize;
  final double toolbarIconSize;
  final double colorIndicatorWidth;
  final double colorDotSize;
  final double searchBoxHeight;
  final double cardTypeIconContainerSize;
}

class AppThemeIcons {
  const AppThemeIcons({
    required this.text,
    required this.image,
    required this.link,
    required this.file,
    required this.folder,
    required this.audio,
    required this.video,
    required this.unknown,
    required this.pin,
    required this.pinFilled,
    required this.delete,
    required this.edit,
    required this.copy,
    required this.paste,
    required this.search,
    required this.filter,
    required this.close,
    required this.settings,
    required this.help,
    required this.recent,
    required this.clear,
    required this.warning,
    required this.colorLabel,
  });

  final IconData text;
  final IconData image;
  final IconData link;
  final IconData file;
  final IconData folder;
  final IconData audio;
  final IconData video;
  final IconData unknown;
  final IconData pin;
  final IconData pinFilled;
  final IconData delete;
  final IconData edit;
  final IconData copy;
  final IconData paste;
  final IconData search;
  final IconData filter;
  final IconData close;
  final IconData settings;
  final IconData help;
  final IconData recent;
  final IconData clear;
  final IconData warning;
  final IconData colorLabel;

  IconData forContentType(int typeValue) => switch (typeValue) {
    0 => text,
    1 => image,
    2 => file,
    3 => folder,
    4 => link,
    5 => audio,
    6 => video,
    _ => unknown,
  };
}

class AppThemeCardStyle {
  const AppThemeCardStyle({
    required this.elevation,
    required this.hoverElevation,
    required this.borderWidth,
    required this.colorIndicatorBorderRadius,
    required this.contentLineHeight,
    required this.headerOpacity,
    required this.footerOpacity,
    required this.timestampOpacity,
    required this.contentOpacity,
    required this.hoverActionOpacity,
    required this.appSourceOpacity,
  });

  final double elevation;
  final double hoverElevation;
  final double borderWidth;
  final BorderRadius colorIndicatorBorderRadius;
  final double contentLineHeight;
  final double headerOpacity;
  final double footerOpacity;
  final double timestampOpacity;
  final double contentOpacity;
  final double hoverActionOpacity;
  final double appSourceOpacity;
}

class AppThemeFilterStyle {
  const AppThemeFilterStyle({
    required this.chipSpacing,
    required this.chipPadding,
    required this.selectedOpacity,
    required this.unselectedOpacity,
    required this.animationDuration,
  });

  final double chipSpacing;
  final EdgeInsets chipPadding;
  final double selectedOpacity;
  final double unselectedOpacity;
  final Duration animationDuration;
}

class AppThemeSearchStyle {
  const AppThemeSearchStyle({
    required this.debounceDuration,
    required this.padding,
    required this.iconOpacity,
  });

  final Duration debounceDuration;
  final EdgeInsets padding;
  final double iconOpacity;
}

class AppThemeToolbarStyle {
  const AppThemeToolbarStyle({
    required this.buttonSpacing,
    required this.buttonPadding,
    required this.iconOpacity,
    required this.hoverOpacity,
  });

  final double buttonSpacing;
  final EdgeInsets buttonPadding;
  final double iconOpacity;
  final double hoverOpacity;
}
