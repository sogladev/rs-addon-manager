// Types for the Rust API
export interface FileReport {
  path: string;
  current_size: number | null;  // None if file is missing
  new_size: number;
}

export interface TransactionReport {
  version: string;
  up_to_date_files: FileReport[];
  outdated_files: FileReport[];
  missing_files: FileReport[];
  removed_files: FileReport[];
  total_download_size: number;
  disk_space_change: number;
  base_path: string;
}

export interface Progress {
    current: number;
    elapsed: number;
    file_index: number;
    file_size: number;
    filename: string;
    speed: number;
    total: number;
    total_files: number;
    total_size_downloaded: number;
    total_amount_left: number;
    expected_time_left: number;
}
