export type Report = {
    id: number;
    author_id: number;
    author_name?: string;
    week: number;
    content: string | null;
    date: number;
};
