import { lazy } from "solid-js";

export const routes = {
    path: "/",
    component: lazy(() => import("./layout")),
    children: [
        {
            path: "/",
            component: lazy(() => import("./index")),
        },
        {
            path: "/week",
            component: lazy(() => import("./week/layout")),
            children: [
                {
                    path: "/:week",
                    component: lazy(() => import("./week/[week]")),
                },
            ],
        },
        {
            path: "/user",
            component: lazy(() => import("./user/layout")),
            children: [
                {
                    path: "/:user",
                    component: lazy(() => import("./user/[user]")),
                },
                {
                    path: "/settings",
                    component: lazy(() => import("./user/settings")),
                },
            ],
        },
        {
            path: "/submit",
            component: lazy(() => import("./submit")),
        },
        {
            path: "/admin",
            component: lazy(() => import("./admin/layout")),
            children: [
                {
                    path: "/",
                    component: lazy(() => import("./admin")),
                },
            ],
        },
        {
            path: "/sigtrap",
            component: lazy(() => import("./sigtrap/layout")),
            children: [
                {
                    path: "/401",
                    component: lazy(() => import("./sigtrap/e401")),
                },
                {
                    path: "/403",
                    component: lazy(() => import("./sigtrap/e403")),
                },
                {
                    path: "/404",
                    component: lazy(() => import("./sigtrap/e404")),
                },
                {
                    path: "/418",
                    component: lazy(() => import("./sigtrap/e418")),
                },
                {
                    path: "/500",
                    component: lazy(() => import("./sigtrap/e500")),
                },
                {
                    path: "/502",
                    component: lazy(() => import("./sigtrap/e502")),
                },
            ],
        },
        {
            path: "*",
            component: lazy(() => import("./sigtrap/e404")),
        },
    ],
};
