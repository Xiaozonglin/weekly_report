import type { DateTime } from "luxon";

export type Article = {
    id: number;
    created_at: DateTime;
    updated_at: DateTime;
    title: string;
    path: string[];
    content: string | null;
    publisher_id: number;
    publisher_name?: string;
    enable_comment: boolean;
    weight: number;
    draft: boolean;
    published: boolean;
};
