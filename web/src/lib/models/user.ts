export type User = {
    id: number;
    name: string;
    email: string | null;
    direction: string | null;
    level: number;
    is_banned: boolean;
    is_hidden: boolean;
    is_admin: boolean;
    recent_reports?: number[];
};
