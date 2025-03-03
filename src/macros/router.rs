macro_rules! router {
    // Initialize the router
    (
        $(
            // Nested routes with path and handlers
            {
                $nest_path:expr,
                $(($path:expr, $method:ident($handler:expr))),*
                $(,)?
            }
        )*

        // Service directive
        $(
            service! {
                ($service_path:expr, $service:expr)
            }
        )*

        // Routes to be merged directly with optional fallback
        {
            $(($merge_path:expr, $merge_method:ident($merge_handler:expr))),*
            $(,)?
            $(
                fallback! { $fallback:expr }
            )?
        }
    ) => {
        {
            let mut router = Router::new();

            // Process nested routes
            $(
                router = router.nest(
                    $nest_path,
                    Router::new()
                        $(
                            .route($path, $method($handler))
                        )*
                );
            )*

            // Process service directive
            $(
                router = router.nest_service($service_path, $service);
            )*

            // Process merged routes with optional fallback
            let merged_router = Router::new()
                $(
                    .route($merge_path, $merge_method($merge_handler))
                )*
                $(
                    .fallback_service($fallback)
                )?;

            router = router.merge(merged_router);

            router
        }
    };
}
pub(crate) use router;
