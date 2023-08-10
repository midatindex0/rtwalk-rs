SELECT f.*,
    u.*,
    COUNT(DISTINCT up.id) AS number_of_users_posted,
    COUNT(p.id) AS number_of_posts,
    STRING_AGG(
        up.username,
        ', '
        ORDER BY up.username ASC
    ) AS users_who_posted
FROM forums f
    JOIN users u ON f.owner_id = u.id
    LEFT JOIN posts p ON f.id = p.forum_id
    LEFT JOIN users up ON p.poster_id = up.id
GROUP BY f.id,
    u.id;