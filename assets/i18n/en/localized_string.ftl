shared-photos =
    {$userName} {$photoCount ->
        [one] added a new photo
        *[other] added {$photoCount} new photos
    } to {$userGender ->
        [male] his stream
        [female] her stream
        *[other] their stream
    }.
username = Username:
gender = Gender:
    .male = Male
    .female = Female
    .other = Other
photo-count = Photo count:
locale = Locale:
