package xyz.xiaban.wallpaper.data

/** One way of getting the file, in the order they should be tried. */
sealed interface Attempt {
    /** The provider's own link. Not verifiable against the manifest, so it is checked by what the file really is. */
    class Source(val file: SourceFile) : Attempt

    /** Our stored copy, verified by hash and size. */
    class Copy(val file: CopyFile) : Attempt

    /** The site holds no copy yet: ask the plain endpoint to make one, then verify it. */
    data object AskForCopy : Attempt
}

object DownloadPlan {
    /**
     * The provider's link first, because it is usually closer and the site stays idle; our own copy when that fails.
     * A site that offers no provider link behaves exactly as before: our verified copy only.
     */
    fun attempts(delivery: Delivery): List<Attempt> = buildList {
        delivery.source?.let { add(Attempt.Source(it)) }
        val copy = delivery.copy
        if (copy != null) add(Attempt.Copy(copy)) else if (delivery.source != null) add(Attempt.AskForCopy)
    }
}
