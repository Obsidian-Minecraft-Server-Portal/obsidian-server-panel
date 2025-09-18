interface String {
    /**
     * Compares the given string with another string for equality, ignoring case differences.
     *
     * @param str - The string to compare with.
     * @return True if the strings are equal ignoring case; otherwise, false.
     */
    equalsIgnoreCase(str: string): boolean;
}

String.prototype.equalsIgnoreCase = function (str: string) { return this.toLowerCase() === str.toLowerCase(); };