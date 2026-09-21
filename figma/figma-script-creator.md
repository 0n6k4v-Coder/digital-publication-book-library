# Figma Component Script Creator

## Book Card

This script creates a regular Figma frame, not a Component.

Structure:

```text
Book Card
├── Thumbnail
│   ├── Auto Layout: Vertical
│   ├── Width: 160px FIXED
│   ├── Height: 240px FIXED
│   ├── Padding: 0
│   ├── Child alignment: CENTER / CENTER
│   ├── Corner radius: 8px
│   └── Clips content: ON
│
└── Book Info
    ├── Auto Layout: Vertical
    ├── Width: FILL
    ├── Height: HUG
    ├── Alignment: LEFT / TOP
    │
    └── Book Name
        ├── Font: Inter Regular
        ├── Font size: 21px
        ├── Color: #000000
        ├── Horizontal alignment: LEFT
        └── Vertical alignment: CENTER
```

### Figma Developer Console Script

```javascript
(async () => {
  // ============================================================
  // BOOK CARD
  // ============================================================
  //
  // Book Card
  //   Vertical Auto Layout
  //   Width: HUG
  //   Height: HUG
  //   Gap: 16px
  //
  //   ├── Thumbnail
  //   │   Auto Layout
  //   │   Width: 160 FIXED
  //   │   Height: 240 FIXED
  //   │   Padding: 0
  //   │   Child: CENTER / CENTER
  //   │   Radius: 8px
  //   │   Clips content: ON
  //   │
  //   └── Book Info
  //       Vertical Auto Layout
  //       Width: FILL
  //       Height: HUG
  //       Align: LEFT / TOP
  //
  //       └── Book Name
  //           21px
  //           Black
  //           Text: LEFT / CENTER
  //
  // NOT A COMPONENT
  // ============================================================

  const BOOK_NAME = "Book Name";

  const THUMBNAIL_WIDTH = 160;
  const THUMBNAIL_HEIGHT = 240;
  const THUMBNAIL_RADIUS = 8;

  const CARD_GAP = 16;

  const BOOK_NAME_SIZE = 21;

  const COLOR_BLACK = "#000000";
  const COLOR_THUMBNAIL = "#EAEAEA";


  // ============================================================
  // HELPERS
  // ============================================================

  function hexToRgb(hex) {
    const value = hex.replace("#", "");

    if (!/^[0-9A-Fa-f]{6}$/.test(value)) {
      throw new Error(
        `Invalid color: ${hex}`
      );
    }

    return {
      r: parseInt(value.slice(0, 2), 16) / 255,
      g: parseInt(value.slice(2, 4), 16) / 255,
      b: parseInt(value.slice(4, 6), 16) / 255,
    };
  }


  function solid(hex, opacity = 1) {
    return [
      {
        type: "SOLID",
        color: hexToRgb(hex),
        opacity,
      },
    ];
  }


  // ============================================================
  // FONT
  // ============================================================

  const FONT = {
    family: "Inter",
    style: "Regular",
  };

  await figma.loadFontAsync(FONT);


  // ============================================================
  // BOOK CARD
  // ============================================================

  const bookCard =
    figma.createFrame();

  bookCard.name =
    "Book Card";


  // ------------------------------------------------------------
  // Vertical Auto Layout
  // ------------------------------------------------------------

  bookCard.layoutMode =
    "VERTICAL";


  // Width = HUG
  bookCard.layoutSizingHorizontal =
    "HUG";


  // Height = HUG
  bookCard.layoutSizingVertical =
    "HUG";


  // ------------------------------------------------------------
  // Gap = 16px
  // ------------------------------------------------------------

  bookCard.itemSpacing =
    CARD_GAP;


  // ------------------------------------------------------------
  // Top / Left alignment
  // ------------------------------------------------------------

  bookCard.primaryAxisAlignItems =
    "MIN";

  bookCard.counterAxisAlignItems =
    "MIN";


  // No padding
  bookCard.paddingTop = 0;
  bookCard.paddingRight = 0;
  bookCard.paddingBottom = 0;
  bookCard.paddingLeft = 0;


  // Transparent
  bookCard.fills = [];
  bookCard.strokes = [];


  // ============================================================
  // THUMBNAIL
  // ============================================================

  const thumbnail =
    figma.createFrame();

  thumbnail.name =
    "Thumbnail";


  // ------------------------------------------------------------
  // Thumbnail itself uses Auto Layout
  // ------------------------------------------------------------

  thumbnail.layoutMode =
    "VERTICAL";


  // ------------------------------------------------------------
  // Fixed dimensions
  // ------------------------------------------------------------

  thumbnail.resize(
    THUMBNAIL_WIDTH,
    THUMBNAIL_HEIGHT
  );

  thumbnail.layoutSizingHorizontal =
    "FIXED";

  thumbnail.layoutSizingVertical =
    "FIXED";


  // ------------------------------------------------------------
  // Zero padding
  // ------------------------------------------------------------

  thumbnail.paddingTop = 0;
  thumbnail.paddingRight = 0;
  thumbnail.paddingBottom = 0;
  thumbnail.paddingLeft = 0;


  // ------------------------------------------------------------
  // Center child
  // ------------------------------------------------------------

  thumbnail.primaryAxisAlignItems =
    "CENTER";

  thumbnail.counterAxisAlignItems =
    "CENTER";


  // No internal gap
  thumbnail.itemSpacing = 0;


  // ------------------------------------------------------------
  // 8px corner radius
  // ------------------------------------------------------------

  thumbnail.cornerRadius =
    THUMBNAIL_RADIUS;


  // ------------------------------------------------------------
  // Clip oversized pasted content
  // ------------------------------------------------------------

  thumbnail.clipsContent =
    true;


  // Placeholder background
  thumbnail.fills =
    solid(COLOR_THUMBNAIL);

  thumbnail.strokes = [];


  // Prevent growth
  thumbnail.layoutGrow = 0;


  // Add Thumbnail to Book Card
  bookCard.appendChild(
    thumbnail
  );


  // ============================================================
  // BOOK INFO
  // ============================================================

  const bookInfo =
    figma.createFrame();

  bookInfo.name =
    "Book Info";


  // Vertical Auto Layout
  bookInfo.layoutMode =
    "VERTICAL";


  // Add to Book Card before setting FILL.
  bookCard.appendChild(
    bookInfo
  );


  // ------------------------------------------------------------
  // Width = FILL
  // ------------------------------------------------------------

  bookInfo.layoutSizingHorizontal =
    "FILL";


  // ------------------------------------------------------------
  // Height = HUG
  // ------------------------------------------------------------

  bookInfo.layoutSizingVertical =
    "HUG";


  // ------------------------------------------------------------
  // LEFT / TOP alignment
  // ------------------------------------------------------------

  bookInfo.primaryAxisAlignItems =
    "MIN";

  bookInfo.counterAxisAlignItems =
    "MIN";


  // No padding
  bookInfo.paddingTop = 0;
  bookInfo.paddingRight = 0;
  bookInfo.paddingBottom = 0;
  bookInfo.paddingLeft = 0;


  // No internal gap
  bookInfo.itemSpacing = 0;


  bookInfo.fills = [];
  bookInfo.strokes = [];


  // ============================================================
  // BOOK NAME
  // ============================================================

  const bookName =
    figma.createText();

  bookName.name =
    "Book Name";


  // Font
  bookName.fontName =
    FONT;

  bookName.fontSize =
    BOOK_NAME_SIZE;

  bookName.characters =
    BOOK_NAME;


  // Black
  bookName.fills =
    solid(COLOR_BLACK);


  // ------------------------------------------------------------
  // Add to Auto Layout parent first.
  // This must happen before setting HUG sizing properties.
  // ------------------------------------------------------------

  bookInfo.appendChild(
    bookName
  );


  // HUG sizing
  bookName.layoutSizingHorizontal =
    "HUG";

  bookName.layoutSizingVertical =
    "HUG";


  // Text follows content
  bookName.textAutoResize =
    "WIDTH_AND_HEIGHT";


  // ------------------------------------------------------------
  // Text alignment
  //
  // LEFT horizontally
  // CENTER vertically
  // ------------------------------------------------------------

  bookName.textAlignHorizontal =
    "LEFT";

  bookName.textAlignVertical =
    "CENTER";


  // ============================================================
  // ADD TO PAGE
  // ============================================================

  const page =
    figma.currentPage;

  page.appendChild(
    bookCard
  );


  // ============================================================
  // POSITION
  // ============================================================

  bookCard.x = 0;
  bookCard.y = 0;


  // ============================================================
  // SELECT
  // ============================================================

  page.selection = [
    bookCard
  ];


  // ============================================================
  // ZOOM
  // ============================================================

  figma.viewport.scrollAndZoomIntoView([
    bookCard
  ]);


  // ============================================================
  // DEBUG
  // ============================================================

  console.log(
    "BOOK CARD CREATED",
    {
      bookCard: {
        layoutMode:
          bookCard.layoutMode,

        width:
          bookCard.layoutSizingHorizontal,

        height:
          bookCard.layoutSizingVertical,

        gap:
          bookCard.itemSpacing,
      },

      thumbnail: {
        width:
          thumbnail.width,

        height:
          thumbnail.height,

        radius:
          thumbnail.cornerRadius,

        layoutMode:
          thumbnail.layoutMode,

        horizontal:
          thumbnail.layoutSizingHorizontal,

        vertical:
          thumbnail.layoutSizingVertical,

        clipsContent:
          thumbnail.clipsContent,

        alignment: {
          primary:
            thumbnail.primaryAxisAlignItems,

          counter:
            thumbnail.counterAxisAlignItems,
        },
      },

      bookInfo: {
        layoutMode:
          bookInfo.layoutMode,

        width:
          bookInfo.layoutSizingHorizontal,

        height:
          bookInfo.layoutSizingVertical,

        alignment: {
          primary:
            bookInfo.primaryAxisAlignItems,

          counter:
            bookInfo.counterAxisAlignItems,
        },
      },

      bookName: {
        fontSize:
          bookName.fontSize,

        horizontalAlignment:
          bookName.textAlignHorizontal,

        verticalAlignment:
          bookName.textAlignVertical,
      },
    }
  );


  console.log(
    "Book Card created successfully."
  );
})();
```
