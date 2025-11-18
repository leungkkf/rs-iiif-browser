pub(crate) mod dataset_ext;

pub(crate) mod iiif_present2 {
    use sophia::api::namespace;

    namespace!(
        "http://iiif.io/api/presentation/2#",
        HasStartCanvas,
        HasContentLayer,
        HasParts,
        HasCollections,
        HasManifests,
        hasSequences,
        hasCanvases,
        HasAnnotations,
        hasImageAnnotations,
        HasLists,
        HasRanges,
        MetadataLabels,
        PresentationDate,
        attributionLabel,
        ViewingDirection,
        ViewingHint,
        LeftToRightDirection,
        RightToLeftDirection,
        TopToBottomDirection,
        BottomToTopDirection,
        PagedHint,
        NonPagedHint,
        ContinuousHint,
        IndividualsHint,
        TopHint,
        MultiPartHint,
        FacingPagesHint,
        Sequence
    );
}

pub(crate) mod iiif_present3 {
    use sophia::api::namespace;

    namespace!(
        "http://iiif.io/api/presentation/3#",
        Collection,
        Manifest,
        Canvas,
        Range,
        MetadataEntries,
        requiredStatement,
        Thumbnail,
        NavigationDate,
        AccompanyingCanvas,
        PlaceholderCanvas,
        ViewingDirection,
        LeftToRightDirection,
        RightToLeftDirection,
        TopToBottomDirection,
        BottomToTopDirection,
        Behavior,
        AutoAdvanceHint,
        NoAutoAdvanceHint,
        RepeatHint,
        NoRepeatHint,
        Unordered,
        IndividualsHint,
        ContinuousHint,
        PagedHint,
        FacingPagesHint,
        NonPagedHint,
        MultiPartHint,
        TogetherHint,
        SequenceHint,
        ThumbnailNavHint,
        NoNavHint,
        NoneHint,
        TimeMode,
        TrimMode,
        ScaleMode,
        LoopMode,
        Start,
        Supplementary,
        Structures,
        Annotations,
        Painting,
        Supplementing,
        ContentState,
        Contextualizing
    );
}

pub(crate) mod exif {
    use sophia::api::namespace;

    namespace!("http://www.w3.org/2003/12/exif/ns#", height, width);
}

pub(crate) mod oa {
    use sophia::api::namespace;

    namespace!(
        "http://www.w3.org/ns/oa#",
        Annotation,
        Selector,
        MotivatedBy,
        hasBody,
        HasTarget,
        HasSource,
        HasSelector,
        StyledBy,
        StyleClass,
        Default,
        Item,
        Prefix,
        Suffix,
        Exact
    );
}

pub(crate) mod cnt {
    use sophia::api::namespace;

    namespace!(
        "http://www.w3.org/2011/content#",
        Chars,
        CharacterEncoding,
        Bytes
    );
}

pub(crate) mod dc {
    use sophia::api::namespace;

    namespace!(
        "http://purl.org/dc/elements/1.1/",
        format,
        Language,
        description
    );
}

pub(crate) mod dcterms {
    use sophia::api::namespace;

    namespace!(
        "http://purl.org/dc/terms/",
        IsPartOf,
        Agent,
        rights,
        ConformsTo,
        HasFormat,
        Relation
    );
}

pub(crate) mod doap {
    use sophia::api::namespace;

    namespace!("http://usefulinc.com/ns/doap#", Implements);
}

pub(crate) mod foaf {
    use sophia::api::namespace;

    namespace!("http://xmlns.com/foaf/0.1/", logo, Homepage, thumbnail);
}

pub(crate) mod xsd {
    use sophia::api::namespace;

    namespace!("http://www.w3.org/2001/XMLSchema#", Integer);
}

pub(crate) mod svcs {
    use sophia::api::namespace;

    namespace!("http://rdfs.org/sioc/services#", has_service);
}

pub(crate) mod acts {
    use sophia::api::namespace;

    namespace!(
        "http://www.w3.org/ns/activitystreams#",
        OrderedCollection,
        OrderedCollectionPage,
        Summary,
        Items,
        First,
        Last,
        Next,
        Prev,
        TotalItems,
        StartIndex
    );
}

pub(crate) mod ebu {
    use sophia::api::namespace;

    namespace!(
        "http://www.ebu.ch/metadata/ontologies/ebucore/ebucore#",
        Duration
    );
}

pub(crate) mod schema {
    use sophia::api::namespace;

    namespace!("https://schema.org/", WebAPI, Provider, PotentialAction);
}
