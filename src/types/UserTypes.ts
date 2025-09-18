export interface PermissionFlag {
    id: number;
    name: string;
}

export interface UserData {
    id: string;
    username: string;
    permissions: PermissionFlag[];
    join_date: string;
    last_online: string;
    is_active: boolean;
    needs_password_change: boolean;
}

export interface CreateUserRequest {
    username: string;
    permissions: number[];
}

export interface UpdateUserRequest {
    username?: string;
    permissions?: number[];
    is_active?: boolean;
}


export interface ApiResponse<T = any> {
    message: string;
    data?: T;
    error?: string;
}
