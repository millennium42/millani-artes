create table main.sale_items (
    id BLOB NOT NULL PRIMARY KEY CHECK(typeof(id) = 'blob' AND length(id) = 16),
    sale_id BLOB NOT NULL CHECK(typeof(sale_id) = 'blob' AND length(sale_id) = 16),
    position INTEGER NOT NULL CHECK (position >= 0),
    product_id BLOB CHECK(product_id IS NULL OR (typeof(product_id) = 'blob' AND length(product_id) = 16)),
    description TEXT NOT NULL,
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price_cents INTEGER NOT NULL CHECK (unit_price_cents >= 0),
    discount_cents INTEGER NOT NULL CHECK (discount_cents >= 0),
    total_cents INTEGER NOT NULL CHECK (total_cents >= 0),
    constraint sale_items_sale_id_fkey foreign key (sale_id) references sales (id) on delete no action on update no action,
    constraint sale_items_product_id_fkey foreign key (product_id) references products (id) on delete no action on update no action,
    constraint sale_items_sale_id_position_unique unique (sale_id, position),
    constraint sale_item_total_consistent check (
        case
            when quantity <= 0 or unit_price_cents < 0 or discount_cents < 0 or total_cents < 0 then 0
            when unit_price_cents < discount_cents / quantity then 0
            when unit_price_cents - discount_cents / quantity < total_cents / quantity then 0
            else unit_price_cents - discount_cents / quantity - total_cents / quantity =
                case
                    when discount_cents % quantity = 0 and total_cents % quantity = 0 then 0
                    when discount_cents % quantity = quantity - (total_cents % quantity) then 1
                    else -1
                end
        end
    )
) STRICT;
