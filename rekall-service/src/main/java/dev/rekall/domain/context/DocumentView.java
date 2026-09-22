package dev.rekall.domain.context;

import dev.rekall.domain.Document;
import dev.rekall.domain.DocumentContextMode;

/** A note as a context hands it over: in full, or as a reference a session loads by its anchor. */
public record DocumentView(String title, String kind, String bodyMarkdown, String anchor, DocumentContextMode contextMode) {

    public static DocumentView of(Document document) {
        return new DocumentView(
                document.getTitle(), document.getKind(), document.getBodyMarkdown(), document.anchor(),
                document.getContextMode());
    }

    /** The same note, handed over in full whatever its mode: what loading it by its own anchor gives. */
    public DocumentView inFull() {
        return new DocumentView(title, kind, bodyMarkdown, anchor, DocumentContextMode.FULL);
    }
}
