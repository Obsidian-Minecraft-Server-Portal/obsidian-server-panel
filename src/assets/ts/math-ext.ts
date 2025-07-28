interface Math
{
    /**
     * Clamps a given number within the inclusive range defined by the minimum and maximum values.
     *
     * @param target The number to be clamped.
     * @param min The lower bound of the range.
     * @param max The upper bound of the range.
     * @return The clamped value, which will be within the range of min and max.
     */
    clamp(target: number, min: number, max: number): number;

    /**
     * Converts a number of bytes into a human-readable string representation.
     * The conversion is done using the International System of Units (SI) prefixes.
     * For example, 1000 bytes will be converted to "1.00 kB", and 1048576 bytes will be converted to "1.00 MB".
     * The conversion is done using the following prefixes:
     * - kB (kilobyte) = 1000 bytes
     * - MB (megabyte) = 1000 kB
     * - GB (gigabyte) = 1000 MB
     * - TB (terabyte) = 1000 GB
     * - PB (petabyte) = 1000 TB
     * - EB (exabyte) = 1000 PB
     * - ZB (zettabyte) = 1000 EB
     * - YB (yottabyte) = 1000 ZB
     * @param bytes
     */
    convertToByteString(bytes: number): string;
}

Math.clamp = function (target: number, min: number, max: number): number
{
    return Math.min(Math.max(target, min), max);
};

Math.convertToByteString = function (bytes: number): string
{
    if (bytes < 0) return "0 B";
    if (bytes === 0) return "0 B";

    const units = ["B", "kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    const base = 1000; // SI base

    const i = Math.floor(Math.log(bytes) / Math.log(base));
    const value = bytes / Math.pow(base, i);

    return `${value.toFixed(2)} ${units[i]}`;
};