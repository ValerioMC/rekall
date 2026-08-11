package dev.rekall.engine;

/** The two schemas Rekall owns, and the ownership rule that separates them. */
public final class RekallSchemas {

    /** Liquibase-managed. Holds the meta-model, the DDL log and the documents. */
    public static final String META = "rekall_meta";

    /** Owned exclusively by the DDL engine. Liquibase creates it and never authors anything inside. */
    public static final String DATA = "rekall_data";

    /** Trigger function defined in {@link #META} and attached to every generated table. */
    public static final String SET_UPDATED_AT_FUNCTION = META + ".set_updated_at()";

    private RekallSchemas() {
    }
}
