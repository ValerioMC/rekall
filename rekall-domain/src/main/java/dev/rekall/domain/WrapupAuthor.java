package dev.rekall.domain;

/**
 * Who last wrote a wrapup.
 *
 * <p>Worth a column, because the two are not interchangeable to the person reading it. A wrapup
 * Claude wrote is a description of the code as the model understood it at the end of a session;
 * a wrapup written by hand is a correction of that. Seeing which one is on screen is what tells
 * you whether the next {@code /rk ... wrapup} is about to overwrite your own words.
 */
public enum WrapupAuthor {

    /** Written over MCP, at the end of a session. */
    CLAUDE,

    /** Written or corrected in the console. */
    HAND
}
