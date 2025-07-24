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
}

Math.clamp = function (target: number, min: number, max: number): number
{
    return Math.min(Math.max(target, min), max);
};
