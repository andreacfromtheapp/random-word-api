-- Create an admin user with password: 'unsafe_password'
INSERT INTO
    users (
        username,
        password_hash,
        is_admin
    )
VALUES
    (
        'admin_user',
        '$argon2id$v=19$m=524288,t=2,p=1$aG9sbG93X2tuaWdodA$CfE/X+Kld4C1TpRrkhpt0HZRpxA8lbMTkLs0m7bsYVk',
        TRUE
    );
