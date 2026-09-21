# Figma Component Script Creator

## Table Of Content

* [Variable](#variable)

  * [Table](#variable-table)
  * [Script](#variable-script)
* [Component](#component)

  * [Book Card](#book-card)

---

## Variable

### Variable Table

<a id="variable-table"></a>

| Collection | Variable         | Type    | Light     | Dark      |
| ---------- | ---------------- | ------- | --------- | --------- |
| `Colors`   | `Text / Primary` | `COLOR` | `#000000` | `#FFFFFF` |

### Variable Script

<a id="variable-script"></a>

* Reuses the `Colors` collection when it exists.
* Reuses the `Text / Primary` variable when it exists.
* Reuses the `Light` and `Dark` modes when they exist.
* Updates the defined mode values.
* Throws an error when the existing variable has the wrong type.

```javascript
(async () => {
  const COLLECTION_NAME = "Colors";
  const VARIABLE_NAME = "Text / Primary";
  const VARIABLE_TYPE = "COLOR";

  const LIGHT_MODE = "Light";
  const DARK_MODE = "Dark";

  const LIGHT_COLOR = {
    r: 0,
    g: 0,
    b: 0,
    a: 1,
  };

  const DARK_COLOR = {
    r: 1,
    g: 1,
    b: 1,
    a: 1,
  };

  const collections =
    await figma.variables
      .getLocalVariableCollectionsAsync();

  let collection =
    collections.find(
      item =>
        item.name === COLLECTION_NAME
    );

  if (!collection) {
    collection =
      figma.variables
        .createVariableCollection(
          COLLECTION_NAME
        );
  }

  let lightMode =
    collection.modes.find(
      mode =>
        mode.name === LIGHT_MODE
    );

  if (!lightMode) {
    const firstMode =
      collection.modes[0];

    if (!firstMode) {
      throw new Error(
        `Collection "${COLLECTION_NAME}" has no modes.`
      );
    }

    collection.renameMode(
      firstMode.modeId,
      LIGHT_MODE
    );

    lightMode =
      collection.modes.find(
        mode =>
          mode.modeId === firstMode.modeId
      );
  }

  let darkMode =
    collection.modes.find(
      mode =>
        mode.name === DARK_MODE
    );

  if (!darkMode) {
    const darkModeId =
      collection.addMode(
        DARK_MODE
      );

    darkMode =
      collection.modes.find(
        mode =>
          mode.modeId === darkModeId
      );
  }

  if (!lightMode || !darkMode) {
    throw new Error(
      "Could not resolve Light and Dark modes."
    );
  }

  const variables =
    await figma.variables
      .getLocalVariablesAsync();

  let variable =
    variables.find(
      item =>
        item.variableCollectionId ===
          collection.id &&
        item.name ===
          VARIABLE_NAME
    );

  if (!variable) {
    variable =
      figma.variables.createVariable(
        VARIABLE_NAME,
        collection,
        VARIABLE_TYPE
      );
  }

  if (
    variable.resolvedType !==
    VARIABLE_TYPE
  ) {
    throw new Error(
      `Variable "${VARIABLE_NAME}" exists with type "${variable.resolvedType}", expected "${VARIABLE_TYPE}".`
    );
  }

  variable.setValueForMode(
    lightMode.modeId,
    LIGHT_COLOR
  );

  variable.setValueForMode(
    darkMode.modeId,
    DARK_COLOR
  );

  console.log(
    "VARIABLE CREATED / UPDATED",
    {
      collection: COLLECTION_NAME,
      variable: VARIABLE_NAME,
      type: variable.resolvedType,

      modes: {
        [LIGHT_MODE]: "#000000",
        [DARK_MODE]: "#FFFFFF",
      },

      ids: {
        collection: collection.id,
        variable: variable.id,
        lightMode: lightMode.modeId,
        darkMode: darkMode.modeId,
      },
    }
  );
})();
```

---

## Component

<a id="component"></a>

### Book Card

<a id="book-card"></a>

Structure

```text
Book Card
├── Thumbnail
│   ├── Auto Layout: Vertical
│   ├── Width: 160 FIXED
│   ├── Height: 240 FIXED
│   ├── Padding: 0
│   ├── Alignment: CENTER / CENTER
│   ├── Corner radius: 8px
│   ├── Clips content: ON
│   └── Fill: None
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
        ├── Color: Colors / Text / Primary
        ├── Horizontal alignment: LEFT
        └── Vertical alignment: CENTER
```
Table

| Element     | Layout               | Width     | Height    | Gap / Padding | Alignment       | Fill                    |
| ----------- | -------------------- | --------- | --------- | ------------- | --------------- | ----------------------- |
| `Book Card` | Vertical Auto Layout | HUG       | HUG       | 16px gap      | Left / Top      | None                    |
| `Thumbnail` | Vertical Auto Layout | 160 FIXED | 240 FIXED | 0 padding     | Center / Center | None                    |
| `Book Info` | Vertical Auto Layout | FILL      | HUG       | 0 padding     | Left / Top      | None                    |
| `Book Name` | Text                 | HUG       | HUG       | —             | Left / Center   | Colors / Text / Primary |

### Figma Developer Console Script

```javascript
(async () => {
  const BOOK_NAME = "Book Name";

  const THUMBNAIL_WIDTH = 160;
  const THUMBNAIL_HEIGHT = 240;
  const THUMBNAIL_RADIUS = 8;

  const CARD_GAP = 16;

  const BOOK_NAME_SIZE = 21;

  const COLOR_COLLECTION = "Colors";
  const TEXT_VARIABLE = "Text / Primary";


  // ============================================================
  // FONT
  // ============================================================

  const FONT = {
    family: "Inter",
    style: "Regular",
  };

  await figma.loadFontAsync(FONT);


  // ============================================================
  // VARIABLE
  // ============================================================

  async function getTextVariable() {
    const collections =
      await figma.variables
        .getLocalVariableCollectionsAsync();

    const collection =
      collections.find(
        item =>
          item.name === COLOR_COLLECTION
      );

    if (!collection) {
      throw new Error(
        `Variable collection "${COLOR_COLLECTION}" was not found.`
      );
    }

    const variables =
      await figma.variables
        .getLocalVariablesAsync();

    const variable =
      variables.find(
        item =>
          item.variableCollectionId ===
            collection.id &&
          item.name === TEXT_VARIABLE
      );

    if (!variable) {
      throw new Error(
        `Variable "${COLOR_COLLECTION} / ${TEXT_VARIABLE}" was not found.`
      );
    }

    if (
      variable.resolvedType !==
      "COLOR"
    ) {
      throw new Error(
        `Variable "${COLOR_COLLECTION} / ${TEXT_VARIABLE}" must be COLOR, found "${variable.resolvedType}".`
      );
    }

    return variable;
  }

  const textVariable =
    await getTextVariable();


  // ============================================================
  // BOOK CARD
  // ============================================================

  const bookCard =
    figma.createFrame();

  bookCard.name =
    "Book Card";

  bookCard.layoutMode =
    "VERTICAL";

  bookCard.layoutSizingHorizontal =
    "HUG";

  bookCard.layoutSizingVertical =
    "HUG";

  bookCard.itemSpacing =
    CARD_GAP;

  bookCard.primaryAxisAlignItems =
    "MIN";

  bookCard.counterAxisAlignItems =
    "MIN";

  bookCard.paddingTop = 0;
  bookCard.paddingRight = 0;
  bookCard.paddingBottom = 0;
  bookCard.paddingLeft = 0;

  bookCard.fills = [];
  bookCard.strokes = [];


  // ============================================================
  // THUMBNAIL
  // ============================================================

  const thumbnail =
    figma.createFrame();

  thumbnail.name =
    "Thumbnail";

  thumbnail.layoutMode =
    "VERTICAL";

  thumbnail.resize(
    THUMBNAIL_WIDTH,
    THUMBNAIL_HEIGHT
  );

  thumbnail.layoutSizingHorizontal =
    "FIXED";

  thumbnail.layoutSizingVertical =
    "FIXED";

  thumbnail.paddingTop = 0;
  thumbnail.paddingRight = 0;
  thumbnail.paddingBottom = 0;
  thumbnail.paddingLeft = 0;

  thumbnail.primaryAxisAlignItems =
    "CENTER";

  thumbnail.counterAxisAlignItems =
    "CENTER";

  thumbnail.itemSpacing = 0;

  thumbnail.cornerRadius =
    THUMBNAIL_RADIUS;

  thumbnail.clipsContent =
    true;

  thumbnail.fills = [];
  thumbnail.strokes = [];

  thumbnail.layoutGrow = 0;

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

  bookInfo.layoutMode =
    "VERTICAL";

  bookCard.appendChild(
    bookInfo
  );

  bookInfo.layoutSizingHorizontal =
    "FILL";

  bookInfo.layoutSizingVertical =
    "HUG";

  bookInfo.primaryAxisAlignItems =
    "MIN";

  bookInfo.counterAxisAlignItems =
    "MIN";

  bookInfo.paddingTop = 0;
  bookInfo.paddingRight = 0;
  bookInfo.paddingBottom = 0;
  bookInfo.paddingLeft = 0;

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

  bookName.fontName =
    FONT;

  bookName.fontSize =
    BOOK_NAME_SIZE;

  bookName.characters =
    BOOK_NAME;

  bookInfo.appendChild(
    bookName
  );

  bookName.layoutSizingHorizontal =
    "HUG";

  bookName.layoutSizingVertical =
    "HUG";

  bookName.textAutoResize =
    "WIDTH_AND_HEIGHT";

  bookName.textAlignHorizontal =
    "LEFT";

  bookName.textAlignVertical =
    "CENTER";


  // ============================================================
  // TEXT COLOR VARIABLE
  // ============================================================

  const fills =
    bookName.fills;

  const boundFills =
    fills.map(
      fill => {

        if (
          fill.type !== "SOLID"
        ) {
          return fill;
        }

        return figma.variables
          .setBoundVariableForPaint(
            fill,
            "color",
            textVariable
          );
      }
    );

  bookName.fills =
    boundFills;


  // ============================================================
  // ADD TO PAGE
  // ============================================================

  const page =
    figma.currentPage;

  page.appendChild(
    bookCard
  );

  bookCard.x = 0;
  bookCard.y = 0;

  page.selection = [
    bookCard
  ];

  figma.viewport
    .scrollAndZoomIntoView([
      bookCard
    ]);


  // ============================================================
  // DONE
  // ============================================================

  console.log(
    "Book Card created successfully.",
    {
      textVariable:
        `${COLOR_COLLECTION} / ${TEXT_VARIABLE}`,

      thumbnail:
        `${THUMBNAIL_WIDTH} × ${THUMBNAIL_HEIGHT}`,

      radius:
        THUMBNAIL_RADIUS,

      gap:
        CARD_GAP,
    }
  );
})();
```
